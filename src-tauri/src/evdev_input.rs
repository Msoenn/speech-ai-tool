//! Linux global-keyboard input via evdev.
//!
//! Reading `/dev/input/event*` is a *passive* operation: the compositor still
//! receives every key, so typing is unaffected. This is the only way to get
//! global key events on Wayland, where the X11 XRecord mechanism (`rdev`) only
//! fires while the app window is focused.

use evdev::KeyCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use evdev::{uinput::VirtualDevice, AttributeSet, EventType, InputEvent};
use crate::output::PasteModifier;

/// Map a Linux input keycode (evdev) to the `rdev::Key` used throughout the app.
/// Mirrors the keys accepted by `hotkey::parse_hotkey_string`.
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
        // Letter keycodes follow the QWERTY layout, not alphabetical order
        // (KEY_A=30, KEY_B=48, …), so map each letter explicitly.
        return Ok(match c {
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
            _ => unreachable!("guarded by 'a'..='z' range"),
        });
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

/// Every keycode the paste injector can emit, declared to uinput. The Linux
/// input core silently drops `EV_KEY` events whose code is absent from the
/// device's keybit, so this MUST cover everything `char_to_keycode` and
/// `paste_modifier_keycode` can return.
fn paste_keycapabilities() -> AttributeSet<KeyCode> {
    let mut keys: AttributeSet<KeyCode> = AttributeSet::new();
    for c in 'a'..='z' {
        if let Ok(code) = char_to_keycode(c) {
            keys.insert(code);
        }
    }
    for c in '0'..='9' {
        if let Ok(code) = char_to_keycode(c) {
            keys.insert(code);
        }
    }
    for c in [' ', '-', '=', '[', ']', '\\', ';', '\'', '`', ',', '.', '/'] {
        if let Ok(code) = char_to_keycode(c) {
            keys.insert(code);
        }
    }
    for m in [PasteModifier::Control, PasteModifier::Shift, PasteModifier::Alt, PasteModifier::Meta] {
        keys.insert(paste_modifier_keycode(m));
    }
    keys
}

/// Lazily create the virtual keyboard on `/dev/uinput` and keep it alive.
fn get_virtual_device() -> Result<MutexGuard<'static, Option<VirtualDevice>>, String> {
    static DEVICE: OnceLock<Mutex<Option<VirtualDevice>>> = OnceLock::new();
    let mtx = DEVICE.get_or_init(|| Mutex::new(None));
    let mut guard = mtx.lock().unwrap();
    if guard.is_none() {
        let keys = paste_keycapabilities();
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
    for (_, mut device) in evdev::enumerate() {
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
                        let Ok(mut cb) = cb.lock() else { continue };
                        (cb)(event_type);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::PasteModifier;

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

    #[test]
    fn capabilities_cover_every_emittable_key() {
        let caps = paste_keycapabilities();
        for c in ('a'..='z').chain('0'..='9') {
            if let Ok(code) = char_to_keycode(c) {
                assert!(caps.contains(code), "no capability for key char {}", c);
            }
        }
        for c in [' ', '-', '=', '[', ']', '\\', ';', '\'', '`', ',', '.', '/'] {
            if let Ok(code) = char_to_keycode(c) {
                assert!(caps.contains(code), "no capability for punctuation {}", c);
            }
        }
        for m in [PasteModifier::Control, PasteModifier::Shift, PasteModifier::Alt, PasteModifier::Meta] {
            assert!(caps.contains(paste_modifier_keycode(m)), "no capability for modifier");
        }
    }
}
