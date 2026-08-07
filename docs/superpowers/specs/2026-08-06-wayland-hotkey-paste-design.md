# Design: Native Wayland support for hotkey + auto-paste

Date: 2026-08-06
Status: Approved (design), pending implementation plan

## Problem

After a reboot the user's Ubuntu session switched from X11 to Wayland
(GNOME Wayland is the system default; no `custom.conf` override exists).
Two features break under Wayland:

1. **Global hotkey** — `hotkey.rs` uses `rdev::listen`, which on Linux is
   implemented with X11 `XRecord` (`rdev-0.5.3/src/linux/listen.rs`). Under
   Wayland the app runs via XWayland, so XRecord only sees keys when the
   app's own window is focused. Result: the hotkey works only while the
   window is focused.
2. **Auto-paste** — `output.rs::press_paste_chord` uses `enigo` 0.2, which
   on Linux injects via X11 XTEST. XTEST events go through XWayland and only
   reach other XWayland apps; they do not reach native Wayland apps
   (most GNOME apps, Firefox, etc.). Auto-paste defaults to on
   (`settings.rs:72`).

The system will stay on Wayland (it is already the default). The app must
work natively on Wayland.

## Approach (approved)

Add the pure-Rust `evdev` crate and use it for both halves on Linux:

- **Listen**: passively read `/dev/input/event*` (no `EVIOCGRAB`, so the
  compositor still receives keys). Works on both Wayland and X11.
- **Inject**: create a virtual keyboard via `/dev/uinput` (`UInputDevice`)
  and emit the paste chord. Works on native Wayland, XWayland, and X11.

Fallback posture (approved): try evdev first; if no readable keyboard
devices (user not in the `input` group), fall back to the existing
`rdev::listen` X11 path. X11 users who never joined `input` keep working
with zero setup. Wayland users must join the `input` group
(`sudo usermod -aG input $USER` + logout).

`enigo` remains the paste backend on macOS and Windows; Linux switches to
uinput. No settings or hotkey-string formats change.

## Architecture

### Dependency (Linux only)

`Cargo.toml`, under `[target.'cfg(target_os = "linux")'.dependencies]`:

```toml
evdev = "0.13"   # current as of 2026-08; pure Rust, no system libs
```

### New module `src-tauri/src/evdev_input.rs` (Linux-only)

Two public functions, each feeding the existing pipeline.

#### Listener: `pub fn listen(callback: impl FnMut(rdev::EventType) + Send + 'static) -> Result<(), String>`

- Enumerate `/dev/input/event*` via `evdev::enumerate()`; keep devices that
  report `EV_KEY` with keyboard keycodes (check `supported_keys()` for a
  representative letter key, e.g. `Key::KEY_A`).
- Open each device, set non-blocking, spawn one reader thread per device.
- Each thread parses `InputEvent`s; for `EV_KEY` events:
  - `value == 1` → `KeyPress`, `value == 0` → `KeyRelease`, `value == 2`
    (autorepeat) → ignored.
  - Map the evdev keycode to `rdev::Key` via
    `fn evdev_to_rdev_key(evdev::Key) -> Option<rdev::Key>`.
  - Call `callback` with `rdev::EventType::KeyPress/KeyRelease`.
- Return `Err` only if no readable keyboard device could be opened.

#### Key mapping

`evdev_to_rdev_key` covers the same keys `parse_hotkey_string` supports:
modifiers (left/right Ctrl/Shift/Alt/Meta), A–Z, 0–9, F1–F12, space,
enter, tab, esc, backspace, delete, insert, home, end, page up/down,
arrows, bracket/slash/semicolon/quote/backquote/comma/dot/minus/equal,
numpad 0–9 and operators, caps lock. Right Alt (`KEY_RIGHTALT`) maps to
`rdev::Key::AltGr` (the user's current hotkey).

#### Injector: `pub fn send_paste_chord(modifiers: &[PasteModifier], key: char) -> Result<(), String>`

- Lazily create a virtual keyboard once with `UInputDevice::create()`,
  stored in a process-wide `Mutex<Option<UInputDevice>>`; declare the
  `KEY_*` capabilities needed.
- Emit (with ~15ms sleep between press and release):
  1. each modifier press (`KEY_LEFTCTRL` etc.),
  2. the character key down (`KEY_A`…`KEY_Z`, `KEY_0`…`KEY_9`, or
     punctuation keycodes),
  3. character key up,
  4. modifiers up in reverse order.
- Return `Err` if `/dev/uinput` is unavailable/unwritable.

#### Self-injection suppression

A shared `AtomicBool` (`self_injecting`) is set for the duration of an
injection; the listener threads skip `EV_KEY` events while it is set.
Without this, a user with a single-key hotkey that collides with a paste
key (e.g. hotkey `V`) would self-trigger on every auto-paste.

### `hotkey.rs`

- `ensure_listener`, on Linux: try `evdev_input::listen` first; on `Err`,
  `eprintln!` the reason and fall back to the existing `rdev::listen` path.
- The event handler must be shared across evdev's device threads and the
  rdev path. Refactor the per-thread `held_keys: HashSet<Key>` closure into
  an `Arc<Mutex<Box<dyn FnMut(rdev::EventType) + Send>>>` that both backends
  invoke. `check_combo`, `HotkeyState`, `parse_hotkey_string`, and the
  `combo_active` edge-triggering are unchanged.
- Keep the `listener_running` idempotency/retry semantics.

### `output.rs`

- `parse_paste_shortcut` returns a platform-neutral
  `(Vec<PasteModifier>, char)` instead of `enigo::Key`. Define
  `enum PasteModifier { Control, Shift, Alt, Meta }` (macOS path maps these
  to `enigo::Key` as today).
- `press_paste_chord` splits by `#[cfg]`:
  - macOS: unchanged (enigo + main-thread marshalling for the TSM crash).
  - Windows: unchanged (enigo).
  - Linux: `evdev_input::send_paste_chord`.
- Keep the existing graceful failure in `copy_and_paste`
  (`"Auto-paste failed (text is in clipboard)"`); on Linux, if uinput is
  unavailable, log the reason.

### Error visibility

If both evdev and the rdev fallback fail to start, emit a `pipeline-status`
event (or a dedicated status) so the dashboard can show a message like
"Hotkey unavailable — add your user to the `input` group", rather than only
an `eprintln`.

## Testing

- Unit tests (no hardware needed):
  - `evdev_to_rdev_key` round-trips the keys used by `parse_hotkey_string`.
  - `parse_paste_shortcut` returns the correct neutral form for
    `"Ctrl+Shift+V"`, `"Cmd+V"`, and rejects unknown modifiers.
- Manual verification on the user's Wayland box:
  1. Right Alt fires with the app window unfocused (recording starts).
  2. Auto-paste text lands in a native Wayland app (e.g. GNOME Text Editor).
  3. No self-trigger when the hotkey is a single key that collides with a
     paste key.
  4. (Optional) X11 session still works via the rdev fallback when evdev is
     unavailable.
- Real `/dev/input` reads cannot be unit-tested in CI; documented as manual.

## Docs

- `README.md`: add a Linux setup note — Wayland users need
  `sudo usermod -aG input $USER` then re-login; X11 users work without it
  (evdev fallback to `rdev`).

## Out of scope

- XDG Desktop Portal `GlobalShortcuts` (approval dialogs, inconsistent
  GNOME support, does not help injection).
- rdev's `unstable_grab` (grabs devices, swallows input — wrong tool).
- Session detection: none needed — evdev + uinput work on both display
  servers.
