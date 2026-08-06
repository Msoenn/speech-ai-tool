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
