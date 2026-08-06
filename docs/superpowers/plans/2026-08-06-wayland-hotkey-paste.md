# Native Wayland Hotkey + Paste Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the global hotkey and auto-paste work natively on GNOME Wayland by replacing the X11-only `rdev` listener and `enigo` XTEST injector with an evdev-based listener and uinput-based injector.

**Architecture:** Add the pure-Rust `evdev` crate (Linux-only). A new `evdev_input` module reads `/dev/input/event*` **passively** (compositor still receives keys — listening, not stealing) and maps evdev keycodes to the app's existing `rdev::Key` type, feeding the unchanged `check_combo` logic. Paste injects the chord through a `VirtualDevice` on `/dev/uinput` (works on native Wayland apps, XWayland apps, and X11). If evdev can't open any keyboard device (user lacks the `input` group), `ensure_listener` falls back to the existing `rdev::listen` X11 path. A shared `SELF_INJECTING` atomic prevents the listener from reacting to the app's own injected keys. No session detection is needed — evdev + uinput work on both display servers.

**Tech Stack:** Rust, Tauri v2, `evdev` 0.13.2, `rdev` 0.5 (kept for its `Key` enum and as X11 fallback), `enigo` 0.2 (macOS/Windows only).

## Global Constraints

- `evdev = "0.13.2"` lives under `[target.'cfg(target_os = "linux")'.dependencies]` in `src-tauri/Cargo.toml` (already done — verify at Task 1).
- New module `src-tauri/src/evdev_input.rs` is registered in `lib.rs` under `#[cfg(target_os = "linux")]`.
- `rdev::Key` remains the canonical hotkey-key type. Settings strings and hotkey formats do NOT change.
- The evdev listener is passive: never call `EVIOCGRAB`. The compositor must still receive every key.
- macOS and Windows paste paths keep using `enigo` unchanged.
- Linux listener order: try `evdev_input::listen` first; on `Err`, fall back to `rdev::listen` (X11 path) and, on Wayland, surface a user-facing warning.
- The listener must ignore the app's own injected keys (shared `SELF_INJECTING` atomic).
- `XDG_SESSION_TYPE=wayland` env var may be read for warning wording only — never to switch code paths.
- All Linux-only code is gated with `#[cfg(target_os = "linux")]`; the crate must still compile on macOS and Windows.

---

### Task 1: Module scaffold + evdev→rdev key mapping

**Files:**
- Modify: `src-tauri/Cargo.toml:38-42` — verify `evdev = "0.13.2"` is under `[target.'cfg(target_os = "linux")'.dependencies]` (it is: after `gdk = "0.18"`). If it's in the top-level `[dependencies]`, move it.
- Create: `src-tauri/src/evdev_input.rs`
- Modify: `src-tauri/src/lib.rs` — add `#[cfg(target_os = "linux")] mod evdev_input;` next to the other `mod` declarations (near `mod hotkey;`, ~line 4).

**Interfaces:**
- Produces: `pub(crate) fn evdev_to_rdev_key(code: u16) -> Option<rdev::Key>` — used by Task 4's listener.

- [ ] **Step 1: Write the failing tests**

`src-tauri/src/evdev_input.rs`:

```rust
//! Linux global-keyboard input via evdev.
//!
//! Reading `/dev/input/event*` is a *passive* operation: the compositor still
//! receives every key, so typing is unaffected. This is the only way to get
//! global key events on Wayland, where the X11 XRecord mechanism (`rdev`) only
//! fires while the app window is focused.

use evdev::KeyCode;

/// Map a Linux input keycode (evdev) to the `rdev::Key` used throughout the app.
/// Mirrors the keys accepted by `hotkey::parse_hotkey_string`.
pub(crate) fn evdev_to_rdev_key(code: u16) -> Option<rdev::Key> {
    todo!("Task 1 Step 3")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_right_alt_to_altgr() {
        assert_eq!(
            evdev_to_rdev_key(KeyCode::KEY_RIGHTALT.0),
            Some(rdev::Key::AltGr)
        );
    }

    #[test]
    fn maps_every_hotkey_key() {
        // Every key parse_hotkey_string accepts must be reachable from evdev.
        let codes = [
            KeyCode::KEY_LEFTCTRL.0, KeyCode::KEY_RIGHTCTRL.0,
            KeyCode::KEY_LEFTSHIFT.0, KeyCode::KEY_RIGHTSHIFT.0,
            KeyCode::KEY_LEFTALT.0, KeyCode::KEY_RIGHTALT.0,
            KeyCode::KEY_LEFTMETA.0, KeyCode::KEY_RIGHTMETA.0,
            KeyCode::KEY_SPACE.0, KeyCode::KEY_ENTER.0, KeyCode::KEY_TAB.0,
            KeyCode::KEY_ESC.0, KeyCode::KEY_BACKSPACE.0, KeyCode::KEY_DELETE.0,
            KeyCode::KEY_INSERT.0, KeyCode::KEY_HOME.0, KeyCode::KEY_END.0,
            KeyCode::KEY_PAGEUP.0, KeyCode::KEY_PAGEDOWN.0, KeyCode::KEY_CAPSLOCK.0,
            KeyCode::KEY_UP.0, KeyCode::KEY_DOWN.0, KeyCode::KEY_LEFT.0, KeyCode::KEY_RIGHT.0,
            KeyCode::KEY_F1.0, KeyCode::KEY_F12.0,
            KeyCode::KEY_0.0, KeyCode::KEY_1.0, KeyCode::KEY_9.0,
            KeyCode::KEY_A.0, KeyCode::KEY_Z.0,
            KeyCode::KEY_MINUS.0, KeyCode::KEY_EQUAL.0, KeyCode::KEY_LEFTBRACE.0,
            KeyCode::KEY_RIGHTBRACE.0, KeyCode::KEY_BACKSLASH.0, KeyCode::KEY_SEMICOLON.0,
            KeyCode::KEY_APOSTROPHE.0, KeyCode::KEY_GRAVE.0, KeyCode::KEY_COMMA.0,
            KeyCode::KEY_DOT.0, KeyCode::KEY_SLASH.0,
            KeyCode::KEY_KP0.0, KeyCode::KEY_KP9.0, KeyCode::KEY_KPDOT.0,
            KeyCode::KEY_KPPLUS.0, KeyCode::KEY_KPMINUS.0, KeyCode::KEY_KPASTERISK.0,
            KeyCode::KEY_KPSLASH.0, KeyCode::KEY_KPENTER.0,
        ];
        for code in codes {
            assert!(evdev_to_rdev_key(code).is_some(), "unmapped evdev code {}", code);
        }
    }

    #[test]
    fn unknown_code_is_none() {
        assert_eq!(evdev_to_rdev_key(0xFFFF), None);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p speech-ai-tool evdev --manifest-path src-tauri/Cargo.toml`
Expected: FAIL — compile error `todo!()`/no mapping, plus `evdev_input` not found unless the `mod` line was added. (Add the `mod evdev_input;` line in `lib.rs` first so the module compiles.)

- [ ] **Step 3: Implement the mapping**

Replace the `todo!()` body:

```rust
pub(crate) fn evdev_to_rdev_key(code: u16) -> Option<rdev::Key> {
    use rdev::Key;
    match KeyCode::new(code) {
        // Modifiers
        KeyCode::KEY_LEFTCTRL => Some(Key::ControlLeft),
        KeyCode::KEY_RIGHTCTRL => Some(Key::ControlRight),
        KeyCode::KEY_LEFTSHIFT => Some(Key::ShiftLeft),
        KeyCode::KEY_RIGHTSHIFT => Some(Key::ShiftRight),
        KeyCode::KEY_LEFTALT => Some(Key::Alt),
        KeyCode::KEY_RIGHTALT => Some(Key::AltGr),
        KeyCode::KEY_LEFTMETA => Some(Key::MetaLeft),
        KeyCode::KEY_RIGHTMETA => Some(Key::MetaRight),
        // Editing / navigation
        KeyCode::KEY_BACKSPACE => Some(Key::Backspace),
        KeyCode::KEY_TAB => Some(Key::Tab),
        KeyCode::KEY_ENTER => Some(Key::Return),
        KeyCode::KEY_ESC => Some(Key::Escape),
        KeyCode::KEY_SPACE => Some(Key::Space),
        KeyCode::KEY_INSERT => Some(Key::Insert),
        KeyCode::KEY_DELETE => Some(Key::Delete),
        KeyCode::KEY_HOME => Some(Key::Home),
        KeyCode::KEY_END => Some(Key::End),
        KeyCode::KEY_PAGEUP => Some(Key::PageUp),
        KeyCode::KEY_PAGEDOWN => Some(Key::PageDown),
        KeyCode::KEY_CAPSLOCK => Some(Key::CapsLock),
        // Arrows
        KeyCode::KEY_UP => Some(Key::UpArrow),
        KeyCode::KEY_DOWN => Some(Key::DownArrow),
        KeyCode::KEY_LEFT => Some(Key::LeftArrow),
        KeyCode::KEY_RIGHT => Some(Key::RightArrow),
        // Function keys
        KeyCode::KEY_F1 => Some(Key::F1),
        KeyCode::KEY_F2 => Some(Key::F2),
        KeyCode::KEY_F3 => Some(Key::F3),
        KeyCode::KEY_F4 => Some(Key::F4),
        KeyCode::KEY_F5 => Some(Key::F5),
        KeyCode::KEY_F6 => Some(Key::F6),
        KeyCode::KEY_F7 => Some(Key::F7),
        KeyCode::KEY_F8 => Some(Key::F8),
        KeyCode::KEY_F9 => Some(Key::F9),
        KeyCode::KEY_F10 => Some(Key::F10),
        KeyCode::KEY_F11 => Some(Key::F11),
        KeyCode::KEY_F12 => Some(Key::F12),
        // Number row (evdev: KEY_1=2 … KEY_9=10, KEY_0=11)
        KeyCode::KEY_0 => Some(Key::Num0),
        KeyCode::KEY_1 => Some(Key::Num1),
        KeyCode::KEY_2 => Some(Key::Num2),
        KeyCode::KEY_3 => Some(Key::Num3),
        KeyCode::KEY_4 => Some(Key::Num4),
        KeyCode::KEY_5 => Some(Key::Num5),
        KeyCode::KEY_6 => Some(Key::Num6),
        KeyCode::KEY_7 => Some(Key::Num7),
        KeyCode::KEY_8 => Some(Key::Num8),
        KeyCode::KEY_9 => Some(Key::Num9),
        // Letters
        KeyCode::KEY_A => Some(Key::KeyA),
        KeyCode::KEY_B => Some(Key::KeyB),
        KeyCode::KEY_C => Some(Key::KeyC),
        KeyCode::KEY_D => Some(Key::KeyD),
        KeyCode::KEY_E => Some(Key::KeyE),
        KeyCode::KEY_F => Some(Key::KeyF),
        KeyCode::KEY_G => Some(Key::KeyG),
        KeyCode::KEY_H => Some(Key::KeyH),
        KeyCode::KEY_I => Some(Key::KeyI),
        KeyCode::KEY_J => Some(Key::KeyJ),
        KeyCode::KEY_K => Some(Key::KeyK),
        KeyCode::KEY_L => Some(Key::KeyL),
        KeyCode::KEY_M => Some(Key::KeyM),
        KeyCode::KEY_N => Some(Key::KeyN),
        KeyCode::KEY_O => Some(Key::KeyO),
        KeyCode::KEY_P => Some(Key::KeyP),
        KeyCode::KEY_Q => Some(Key::KeyQ),
        KeyCode::KEY_R => Some(Key::KeyR),
        KeyCode::KEY_S => Some(Key::KeyS),
        KeyCode::KEY_T => Some(Key::KeyT),
        KeyCode::KEY_U => Some(Key::KeyU),
        KeyCode::KEY_V => Some(Key::KeyV),
        KeyCode::KEY_W => Some(Key::KeyW),
        KeyCode::KEY_X => Some(Key::KeyX),
        KeyCode::KEY_Y => Some(Key::KeyY),
        KeyCode::KEY_Z => Some(Key::KeyZ),
        // Punctuation / symbols
        KeyCode::KEY_MINUS => Some(Key::Minus),
        KeyCode::KEY_EQUAL => Some(Key::Equal),
        KeyCode::KEY_LEFTBRACE => Some(Key::LeftBracket),
        KeyCode::KEY_RIGHTBRACE => Some(Key::RightBracket),
        KeyCode::KEY_BACKSLASH => Some(Key::BackSlash),
        KeyCode::KEY_SEMICOLON => Some(Key::SemiColon),
        KeyCode::KEY_APOSTROPHE => Some(Key::Quote),
        KeyCode::KEY_GRAVE => Some(Key::BackQuote),
        KeyCode::KEY_COMMA => Some(Key::Comma),
        KeyCode::KEY_DOT => Some(Key::Dot),
        KeyCode::KEY_SLASH => Some(Key::Slash),
        // Numpad
        KeyCode::KEY_KP0 => Some(Key::Kp0),
        KeyCode::KEY_KP1 => Some(Key::Kp1),
        KeyCode::KEY_KP2 => Some(Key::Kp2),
        KeyCode::KEY_KP3 => Some(Key::Kp3),
        KeyCode::KEY_KP4 => Some(Key::Kp4),
        KeyCode::KEY_KP5 => Some(Key::Kp5),
        KeyCode::KEY_KP6 => Some(Key::Kp6),
        KeyCode::KEY_KP7 => Some(Key::Kp7),
        KeyCode::KEY_KP8 => Some(Key::Kp8),
        KeyCode::KEY_KP9 => Some(Key::Kp9),
        KeyCode::KEY_KPDOT => Some(Key::KpDelete),
        KeyCode::KEY_KPPLUS => Some(Key::KpPlus),
        KeyCode::KEY_KPMINUS => Some(Key::KpMinus),
        KeyCode::KEY_KPASTERISK => Some(Key::KpMultiply),
        KeyCode::KEY_KPSLASH => Some(Key::KpDivide),
        KeyCode::KEY_KPENTER => Some(Key::KpReturn),
        _ => None,
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p speech-ai-tool evdev --manifest-path src-tauri/Cargo.toml`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/evdev_input.rs src-tauri/src/lib.rs
git commit -m "feat(hotkey): add evdev keycode mapping for Wayland global input"
```

---

### Task 2: Platform-neutral paste-shortcut parsing

**Files:**
- Modify: `src-tauri/src/output.rs:35-69` (`parse_paste_shortcut`) and `output.rs:71-135` (`simulate_paste`/`press_paste_chord`)

**Interfaces:**
- Consumes: nothing new.
- Produces: `pub enum PasteModifier { Control, Shift, Alt, Meta }` and `fn parse_paste_shortcut(&str) -> Result<(Vec<PasteModifier>, char), AppError>` — consumed by Task 3 (`evdev_input::send_paste_chord`) and by `press_paste_chord` (this task).

- [ ] **Step 1: Write the failing tests**

Append to `src-tauri/src/output.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ctrl_shift_v() {
        assert_eq!(
            parse_paste_shortcut("Ctrl+Shift+V").unwrap(),
            (vec![PasteModifier::Control, PasteModifier::Shift], 'v')
        );
    }

    #[test]
    fn parses_cmd_v() {
        assert_eq!(
            parse_paste_shortcut("Cmd+V").unwrap(),
            (vec![PasteModifier::Meta], 'v')
        );
    }

    #[test]
    fn rejects_unknown_modifier() {
        assert!(parse_paste_shortcut("Hyper+V").is_err());
    }

    #[test]
    fn rejects_empty_shortcut() {
        assert!(parse_paste_shortcut("").is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p speech-ai-tool output --manifest-path src-tauri/Cargo.toml`
Expected: FAIL — `PasteModifier` not defined, `parse_paste_shortcut` returns `enigo::Key` not `PasteModifier`.

- [ ] **Step 3: Add `PasteModifier` and rework parsing**

In `output.rs`, add the enum near the top (after the imports):

```rust
/// A modifier in a paste shortcut, independent of any backend's key enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteModifier {
    Control,
    Shift,
    Alt,
    Meta,
}
```

Replace the body of `parse_paste_shortcut` (keep its signature's `AppError` result but change the return tuple type):

```rust
/// Parse a shortcut string like "Ctrl+Shift+V" or "Cmd+V" into modifiers + a key.
fn parse_paste_shortcut(shortcut: &str) -> Result<(Vec<PasteModifier>, char), AppError> {
    let parts: Vec<&str> = shortcut.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return Err(AppError::Output("Empty paste shortcut".into()));
    }

    let mut modifiers = Vec::new();
    for part in &parts[..parts.len() - 1] {
        let modifier = match part.to_lowercase().as_str() {
            "ctrl" | "control" => PasteModifier::Control,
            "shift" => PasteModifier::Shift,
            "alt" => PasteModifier::Alt,
            "cmd" | "meta" | "super" => PasteModifier::Meta,
            other => {
                return Err(AppError::Output(format!(
                    "Unknown modifier in paste shortcut: {}",
                    other
                )))
            }
        };
        modifiers.push(modifier);
    }

    let last = parts.last().unwrap();
    let char_key = if last.len() == 1 {
        last.to_lowercase().chars().next().unwrap()
    } else {
        return Err(AppError::Output(format!(
            "Invalid key in paste shortcut: {}",
            last
        )));
    };

    Ok((modifiers, char_key))
}
```

- [ ] **Step 4: Rework `press_paste_chord` to the neutral types**

Replace the existing `press_paste_chord` (currently takes `&[enigo::Key], enigo::Key`) so it takes the neutral `&[PasteModifier], char` and converts internally. It stays fully enigo-based on every platform for now — Task 3 switches the Linux branch to uinput:

```rust
/// Synthesize the paste chord (modifiers + key) via enigo.
fn press_paste_chord(modifiers: &[PasteModifier], char_key: char) -> Result<(), String> {
    use enigo::{Direction, Enigo, Key as EnigoKey, Keyboard, Settings};

    fn enigo_key_for(m: PasteModifier) -> EnigoKey {
        match m {
            PasteModifier::Control => EnigoKey::Control,
            PasteModifier::Shift => EnigoKey::Shift,
            PasteModifier::Alt => EnigoKey::Alt,
            PasteModifier::Meta => EnigoKey::Meta,
        }
    }

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to create enigo: {}", e))?;

    // Defensively release all common modifiers to ensure clean state (e.g. the
    // hotkey's own modifiers may still be physically held).
    let all_modifiers = [EnigoKey::Control, EnigoKey::Shift, EnigoKey::Alt, EnigoKey::Meta];
    for m in &all_modifiers {
        let _ = enigo.key(*m, Direction::Release);
    }
    std::thread::sleep(std::time::Duration::from_millis(50));

    for m in modifiers {
        enigo
            .key(enigo_key_for(*m), Direction::Press)
            .map_err(|e| format!("Key press failed: {}", e))?;
    }

    enigo
        .key(EnigoKey::Unicode(char_key), Direction::Click)
        .map_err(|e| format!("Key click failed: {}", e))?;

    for m in modifiers.iter().rev() {
        enigo
            .key(enigo_key_for(*m), Direction::Release)
            .map_err(|e| format!("Key release failed: {}", e))?;
    }

    Ok(())
}
```

Note: `simulate_paste` (which marshals onto the macOS main thread) already calls `press_paste_chord(&modifiers, char_key)` — with `modifiers: Vec<PasteModifier>` and `char_key: char`, no other call-site changes are needed.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p speech-ai-tool output --manifest-path src-tauri/Cargo.toml`
Expected: PASS (4 tests).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/output.rs
git commit -m "feat(paste): platform-neutral paste-shortcut parsing for Wayland injection"
```

---

### Task 3: uinput injector (`send_paste_chord`) + self-injection flag

**Files:**
- Modify: `src-tauri/src/evdev_input.rs` (append)

**Interfaces:**
- Consumes: `crate::output::PasteModifier` (Task 2).
- Produces: `pub(crate) fn send_paste_chord(modifiers: &[PasteModifier], key: char) -> Result<(), String>` (wired into `press_paste_chord`'s Linux branch in Step 5) and `pub(crate) fn suppressing_self_injection() -> bool` (consumed by Task 4's listener and Task 5's hotkey handler).

- [ ] **Step 1: Write the failing tests**

Append to `src-tauri/src/evdev_input.rs` inside the existing `#[cfg(test)] mod tests`:

```rust
    #[test]
    fn maps_paste_char_to_keycode() {
        assert_eq!(char_to_keycode('v'), Ok(KeyCode::KEY_V));
        assert_eq!(char_to_keycode('0'), Ok(KeyCode::KEY_0));
        assert_eq!(char_to_keycode(';'), Ok(KeyCode::KEY_SEMICOLON));
        assert!(char_to_keycode('!').is_err());
    }

    #[test]
    fn maps_paste_modifier_to_keycode() {
        assert_eq!(paste_modifier_keycode(PasteModifier::Control).0, KeyCode::KEY_LEFTCTRL.0);
        assert_eq!(paste_modifier_keycode(PasteModifier::Meta).0, KeyCode::KEY_LEFTMETA.0);
    }
```

(Import `crate::output::PasteModifier` at the top of the test module, or use the full path.)

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p speech-ai-tool evdev --manifest-path src-tauri/Cargo.toml`
Expected: FAIL — `char_to_keycode`/`paste_modifier_keycode` not defined.

- [ ] **Step 3: Add the injector**

Append to `src-tauri/src/evdev_input.rs` (add these `use` lines at the top of the file first: `use std::sync::atomic::{AtomicBool, Ordering};`, `use std::sync::{Mutex, MutexGuard, OnceLock};`, `use evdev::uinput::VirtualDevice;`, `use evdev::{AttributeSet, EventType, InputEvent};`, `use crate::output::PasteModifier;`):

> Correction (verified against evdev 0.13.2 source during execution): `VirtualDevice` lives at `evdev::uinput::VirtualDevice`, not the crate root. `VirtualDevice::emit` takes `&mut self` (so `send_paste_chord` uses `guard.as_mut()`, not `as_ref()`). Letter keycodes are QWERTY-ordered, not consecutive — `char_to_keycode` maps each letter explicitly below.

```rust
/// Set while we inject our own keystrokes, so the evdev listener ignores them.
static SELF_INJECTING: AtomicBool = AtomicBool::new(false);

/// True while the app is injecting its own keys (paste chord).
pub(crate) fn suppressing_self_injection() -> bool {
    SELF_INJECTING.load(Ordering::Relaxed)
}

/// Map a paste modifier to the evdev keycode for the left-hand variant.
fn paste_modifier_keycode(m: PasteModifier) -> KeyCode {
    match m {
        PasteModifier::Control => KeyCode::KEY_LEFTCTRL,
        PasteModifier::Shift => KeyCode::KEY_LEFTSHIFT,
        PasteModifier::Alt => KeyCode::KEY_LEFTALT,
        PasteModifier::Meta => KeyCode::KEY_LEFTMETA,
    }
}

/// Map a single paste key character to its evdev keycode.
fn char_to_keycode(c: char) -> Result<KeyCode, String> {
    if ('a'..='z').contains(&c) {
        // Letters are QWERTY-ordered in evdev (KEY_A=30, KEY_B=48, …), so map
        // each explicitly rather than by arithmetic.
        let key = match c {
            'a' => KeyCode::KEY_A,
            'b' => KeyCode::KEY_B,
            'c' => KeyCode::KEY_C,
            'd' => KeyCode::KEY_D,
            'e' => KeyCode::KEY_E,
            'f' => KeyCode::KEY_F,
            'g' => KeyCode::KEY_G,
            'h' => KeyCode::KEY_H,
            'i' => KeyCode::KEY_I,
            'j' => KeyCode::KEY_J,
            'k' => KeyCode::KEY_K,
            'l' => KeyCode::KEY_L,
            'm' => KeyCode::KEY_M,
            'n' => KeyCode::KEY_N,
            'o' => KeyCode::KEY_O,
            'p' => KeyCode::KEY_P,
            'q' => KeyCode::KEY_Q,
            'r' => KeyCode::KEY_R,
            's' => KeyCode::KEY_S,
            't' => KeyCode::KEY_T,
            'u' => KeyCode::KEY_U,
            'v' => KeyCode::KEY_V,
            'w' => KeyCode::KEY_W,
            'x' => KeyCode::KEY_X,
            'y' => KeyCode::KEY_Y,
            'z' => KeyCode::KEY_Z,
            _ => unreachable!(),
        };
        return Ok(key);
    }
    if ('0'..='9').contains(&c) {
        // KEY_1(2)…KEY_9(10) are consecutive; KEY_0(11) is separate.
        if c == '0' {
            return Ok(KeyCode::KEY_0);
        }
        return Ok(KeyCode::new(KeyCode::KEY_1.0 + (c as u32 - '1' as u32) as u16));
    }
    let key = match c {
        ' ' => KeyCode::KEY_SPACE,
        '-' => KeyCode::KEY_MINUS,
        '=' => KeyCode::KEY_EQUAL,
        '[' => KeyCode::KEY_LEFTBRACE,
        ']' => KeyCode::KEY_RIGHTBRACE,
        '\\' => KeyCode::KEY_BACKSLASH,
        ';' => KeyCode::KEY_SEMICOLON,
        '\'' => KeyCode::KEY_APOSTROPHE,
        '`' => KeyCode::KEY_GRAVE,
        ',' => KeyCode::KEY_COMMA,
        '.' => KeyCode::KEY_DOT,
        '/' => KeyCode::KEY_SLASH,
        _ => return Err(format!("unsupported paste key: {}", c)),
    };
    Ok(key)
}

/// Lazily create the virtual keyboard on `/dev/uinput` and keep it alive.
fn get_virtual_device() -> Result<MutexGuard<'static, Option<VirtualDevice>>, String> {
    static DEVICE: OnceLock<Mutex<Option<VirtualDevice>>> = OnceLock::new();
    let mtx = DEVICE.get_or_init(|| Mutex::new(None));
    let mut guard = mtx.lock().unwrap();
    if guard.is_none() {
        let keys: AttributeSet<KeyCode> = [
            KeyCode::KEY_LEFTCTRL, KeyCode::KEY_LEFTSHIFT, KeyCode::KEY_LEFTALT, KeyCode::KEY_LEFTMETA,
            KeyCode::KEY_A, KeyCode::KEY_Z,
            KeyCode::KEY_0, KeyCode::KEY_9,
            KeyCode::KEY_SPACE, KeyCode::KEY_MINUS, KeyCode::KEY_EQUAL,
            KeyCode::KEY_LEFTBRACE, KeyCode::KEY_RIGHTBRACE, KeyCode::KEY_BACKSLASH,
            KeyCode::KEY_SEMICOLON, KeyCode::KEY_APOSTROPHE, KeyCode::KEY_GRAVE,
            KeyCode::KEY_COMMA, KeyCode::KEY_DOT, KeyCode::KEY_SLASH,
        ]
        .into_iter()
        .collect();
        let dev = VirtualDevice::builder()
            .map_err(|e| format!("failed to open /dev/uinput: {}", e))?
            .name("speech-ai-tool-paste")
            .with_keys(&keys)
            .map_err(|e| format!("failed to configure uinput device: {}", e))?
            .build()
            .map_err(|e| format!("failed to create uinput device: {}", e))?;
        *guard = Some(dev);
    }
    Ok(guard)
}

/// Inject a paste chord (modifiers + key) through a virtual keyboard. Works on
/// native Wayland apps, XWayland apps, and X11. Sets the self-injection flag
/// so the evdev listener ignores these synthetic events.
pub(crate) fn send_paste_chord(modifiers: &[PasteModifier], key: char) -> Result<(), String> {
    let keycode = char_to_keycode(key)?;
    let mut guard = get_virtual_device()?;
    let device = guard.as_mut().expect("virtual device created on first paste");

    let press: Vec<InputEvent> = modifiers
        .iter()
        .map(|m| InputEvent::new(EventType::KEY.0, paste_modifier_keycode(*m).0, 1))
        .chain(std::iter::once(InputEvent::new(EventType::KEY.0, keycode.0, 1)))
        .collect();
    let release: Vec<InputEvent> = std::iter::once(InputEvent::new(EventType::KEY.0, keycode.0, 0))
        .chain(
            modifiers
                .iter()
                .rev()
                .map(|m| InputEvent::new(EventType::KEY.0, paste_modifier_keycode(*m).0, 0)),
        )
        .collect();

    SELF_INJECTING.store(true, Ordering::Relaxed);
    let result = (|| -> Result<(), String> {
        device
            .emit(&press)
            .map_err(|e| format!("uinput press failed: {}", e))?;
        std::thread::sleep(std::time::Duration::from_millis(15));
        device
            .emit(&release)
            .map_err(|e| format!("uinput release failed: {}", e))?;
        Ok(())
    })();
    // Let in-flight injected events drain while the flag is still set so the
    // listener skips them rather than racing the flag clear.
    std::thread::sleep(std::time::Duration::from_millis(30));
    SELF_INJECTING.store(false, Ordering::Relaxed);
    result
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p speech-ai-tool evdev --manifest-path src-tauri/Cargo.toml`
Expected: PASS (5 tests).

- [ ] **Step 5: Wire `send_paste_chord` into `press_paste_chord`'s Linux branch**

In `src-tauri/src/output.rs`, the `press_paste_chord` function (created in Task 2, still enigo on all platforms) must become a dispatcher. Edit it so the file reads:

```rust
/// Synthesize the paste chord (modifiers + key).
fn press_paste_chord(modifiers: &[PasteModifier], char_key: char) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        crate::evdev_input::send_paste_chord(modifiers, char_key)
    }
    #[cfg(not(target_os = "linux"))]
    {
        press_paste_chord_enigo(modifiers, char_key)
    }
}

/// enigo-based injection for macOS and Windows (kept unchanged in behavior).
#[cfg(not(target_os = "linux"))]
fn press_paste_chord_enigo(modifiers: &[PasteModifier], char_key: char) -> Result<(), String> {
    // Move the entire enigo body from the current `press_paste_chord` here,
    // unchanged — including its `use enigo::{...}` import and the nested
    // `fn enigo_key_for(...)`.
}
```

That is: rename the existing function to `press_paste_chord_enigo`, gate it with `#[cfg(not(target_os = "linux"))]`, and add the two-branch dispatcher above it.

- [ ] **Step 6: Verify the crate compiles on Linux**

Run: `cargo check -p speech-ai-tool --manifest-path src-tauri/Cargo.toml`
Expected: no errors (the Linux branch resolves `crate::evdev_input::send_paste_chord`).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/evdev_input.rs src-tauri/src/output.rs
git commit -m "feat(paste): uinput-based key injection for native Wayland paste"
```

---

### Task 4: evdev listener

**Files:**
- Modify: `src-tauri/src/evdev_input.rs` (append)

**Interfaces:**
- Consumes: `evdev_to_rdev_key` (Task 1), `SELF_INJECTING` (Task 3).
- Produces: `pub(crate) fn listen(callback: impl FnMut(rdev::EventType) + Send + 'static) -> Result<(), String>` (consumed by Task 5).

- [ ] **Step 1: No unit test (device I/O) — verify by compile**

The listener reads `/dev/input/event*` which can't be exercised in a unit test. Its test is: compiles + Task 5's manual end-to-end check.

- [ ] **Step 2: Implement the listener**

Append to `src-tauri/src/evdev_input.rs` (add `use std::sync::Arc;` to the top-of-file imports):

```rust
/// True if the device looks like a keyboard (reports a letter key).
fn is_keyboard(device: &evdev::Device) -> bool {
    device
        .supported_keys()
        .map_or(false, |keys| keys.contains(KeyCode::KEY_A))
}

/// Listen for global key events on all keyboard devices. Each device gets its
/// own reader thread; `fetch_events` blocks, so no polling is needed. Events
/// are mapped to `rdev::EventType` and forwarded to `callback`.
///
/// Returns `Err` only if no readable keyboard device could be opened (e.g. the
/// user is not in the `input` group).
pub(crate) fn listen(
    callback: impl FnMut(rdev::EventType) + Send + 'static,
) -> Result<(), String> {
    let callback: Arc<Mutex<Box<dyn FnMut(rdev::EventType) + Send>>> =
        Arc::new(Mutex::new(Box::new(callback)));

    let mut opened = 0usize;
    for device in evdev::enumerate() {
        let Ok(mut device) = device else { continue };
        if !is_keyboard(&device) {
            continue;
        }
        opened += 1;
        let cb = Arc::clone(&callback);
        std::thread::spawn(move || loop {
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if event.event_type() != EventType::KEY {
                            continue;
                        }
                        if SELF_INJECTING.load(Ordering::Relaxed) {
                            continue;
                        }
                        let Some(key) = evdev_to_rdev_key(event.code()) else {
                            continue;
                        };
                        let event_type = match event.value() {
                            1 => rdev::EventType::KeyPress(key),
                            0 => rdev::EventType::KeyRelease(key),
                            _ => continue, // autorepeat
                        };
                        (cb.lock().unwrap())(event_type);
                    }
                }
                Err(e) => {
                    eprintln!("evdev device read error: {} (device removed?)", e);
                    break;
                }
            }
        });
    }

    if opened == 0 {
        return Err(
            "no readable keyboard devices in /dev/input (is your user in the 'input' group?)"
                .into(),
        );
    }
    Ok(())
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p speech-ai-tool --manifest-path src-tauri/Cargo.toml`
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/evdev_input.rs
git commit -m "feat(hotkey): passive evdev global key listener for Wayland"
```

---

### Task 5: Wire evdev-first listener into `hotkey.rs` with X11 fallback

**Files:**
- Modify: `src-tauri/src/hotkey.rs:45-86` (`ensure_listener`)

**Interfaces:**
- Consumes: `crate::evdev_input::listen` and `crate::evdev_input::suppressing_self_injection` (Tasks 3–4).
- Produces: the existing `ensure_listener(app: &AppHandle, state: &Arc<HotkeyState>)` behavior, now evdev-first on Linux. No signature changes.

- [ ] **Step 1: No unit test (threading + devices) — verify by compile and manual test**

- [ ] **Step 2: Rewrite `ensure_listener`**

Replace the body of `ensure_listener` in `src-tauri/src/hotkey.rs` with:

```rust
pub fn ensure_listener(app: &AppHandle, state: &Arc<HotkeyState>) {
    if state.listener_running.swap(true, Ordering::SeqCst) {
        return; // already running
    }

    let state_clone = Arc::clone(state);
    let running_flag = Arc::clone(state);
    let app_handle = app.clone();

    thread::spawn(move || {
        // Shared event handler: tracks held keys and triggers the combo. It is
        // behind an Arc<Mutex> because the evdev listener runs one thread per
        // keyboard device, all feeding the same handler.
        let held_keys: Arc<Mutex<HashSet<Key>>> = Arc::new(Mutex::new(HashSet::new()));
        let handler: Arc<Mutex<Box<dyn FnMut(EventType) + Send>>> = Arc::new(Mutex::new(Box::new(
            move |event_type: EventType| {
                #[cfg(target_os = "linux")]
                if crate::evdev_input::suppressing_self_injection() {
                    return;
                }
                let mut held = held_keys.lock().unwrap();
                match event_type {
                    EventType::KeyPress(key) => {
                        held.insert(key);
                        check_combo(&held, &state_clone, &app_handle);
                    }
                    EventType::KeyRelease(key) => {
                        held.remove(&key);
                        check_combo(&held, &state_clone, &app_handle);
                    }
                    _ => {}
                }
            },
        )));

        #[cfg(target_os = "macos")]
        {
            crate::macos_event_tap::listen(move |event_type| {
                (handler.lock().unwrap())(event_type);
            });
        }

        #[cfg(target_os = "linux")]
        {
            let is_wayland = std::env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland");
            let evdev_cb = Arc::clone(&handler);
            if let Err(e) = crate::evdev_input::listen(move |et| (evdev_cb.lock().unwrap())(et)) {
                eprintln!("evdev listener unavailable: {}", e);
                // On Wayland the rdev fallback only works while the window is
                // focused, so tell the user what to do rather than fail silently.
                if is_wayland {
                    let _ = app_handle.emit(
                        "pipeline-status",
                        PipelineStatusEvent {
                            status: PipelineStatus::Error,
                            raw_text: None,
                            cleaned_text: None,
                            error: Some(format!(
                                "Global hotkey will only work while this window is focused: \
                                 evdev could not read input devices ({e}). Add your user to the \
                                 'input' group (sudo usermod -aG input $USER), then log out and back in."
                            )),
                        },
                    );
                }
                let rdev_cb = Arc::clone(&handler);
                if let Err(e2) = rdev::listen(move |ev| (rdev_cb.lock().unwrap())(ev.event_type)) {
                    eprintln!("rdev listener error: {:?}", e2);
                }
            }
        }

        #[cfg(all(not(target_os = "macos"), not(target_os = "linux")))]
        {
            let cb = Arc::clone(&handler);
            if let Err(e) = rdev::listen(move |ev| (cb.lock().unwrap())(ev.event_type)) {
                eprintln!("rdev listener error: {:?}", e);
            }
        }

        running_flag.listener_running.store(false, Ordering::SeqCst);
    });
}
```

Notes:
- `PipelineStatus`, `PipelineStatusEvent`, and `Emitter` are already imported at the top of `hotkey.rs` (used by `on_hotkey_pressed`).
- The `macos_event_tap::listen` argument is now a small wrapper around the shared handler; its signature (`impl FnMut(EventType)`) is unchanged.

- [ ] **Step 3: Verify all platforms compile (best-effort)**

Run: `cargo check -p speech-ai-tool --manifest-path src-tauri/Cargo.toml` (Linux, current platform)
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/hotkey.rs
git commit -m "feat(hotkey): evdev-first global listener with X11 fallback"
```

---

### Task 6: README setup note + end-to-end manual verification

**Files:**
- Modify: `README.md` (add a Linux setup subsection)

- [ ] **Step 1: Add the README note**

Add a `### Linux (Wayland)` subsection under any existing Linux/Setup heading in `README.md`:

```markdown
### Linux (Wayland)

Global hotkeys read the raw input devices. On Wayland sessions the app needs
read access to `/dev/input`, which most users don't have by default:

    sudo usermod -aG input $USER

Log out and back in afterwards. X11 sessions work without this step (the app
falls back to the X11 listener automatically).
```

- [ ] **Step 2: Full build**

Run: `cargo build -p speech-ai-tool --manifest-path src-tauri/Cargo.toml`
Expected: builds cleanly.

- [ ] **Step 3: Run the app and verify on Wayland**

Run the app (dev mode: `pnpm tauri dev`, or the built binary). In a Wayland session:

1. Make sure some window other than the app is focused.
2. Press and hold **Right Alt** → recording should start (tray shows "recording", start tone plays). Release → pipeline runs.
3. Focus a native Wayland app (e.g., GNOME "Text Editor"), dictate, and confirm the cleaned text is pasted into it automatically.
4. Confirm typing in any app is completely unaffected (the listener is passive).
5. Optional negative test: set the hotkey to a single letter (e.g., `V`) in Settings, then transcribe so auto-paste runs — the injected Ctrl+V must NOT re-trigger the hotkey (self-injection suppression).

- [ ] **Step 4: Verify X11 fallback still works (optional)**

In an X11 session (or with the user removed from `input`), confirm the app still functions via the `rdev` fallback (or that the warning appears on Wayland when evdev is unavailable).

- [ ] **Step 5: Commit**

```bash
git add README.md
git commit -m "docs: document input group requirement for Wayland hotkeys"
```

---

## Self-Review Checklist (run after writing the plan)

- [ ] Spec coverage: hotkey (Tasks 1, 4, 5), paste (Tasks 2, 3), fallback (Task 5), self-injection (Tasks 3, 5), error visibility (Task 5), README (Task 6), testing (Tasks 1–2 unit + Task 6 manual).
- [ ] Placeholder scan: no TBD/TODO beyond Task 1's deliberate `todo!()` that Step 3 replaces.
- [ ] Type consistency: `PasteModifier` (Task 2) used by `send_paste_chord` (Task 3) and `press_paste_chord` (Task 2); `evdev_to_rdev_key(u16) -> Option<rdev::Key>` (Task 1) used in Task 4; `listen(impl FnMut(rdev::EventType)+Send)` (Task 4) used in Task 5; `suppressing_self_injection()` (Task 3) used in Tasks 4 and 5.
