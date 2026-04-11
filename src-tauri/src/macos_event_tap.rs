//! Direct CGEvent tap for macOS — bypasses rdev's TSMGetInputSourceProperty call
//! which crashes on macOS 26.3+ when called from a background thread.
//!
//! We only need virtual keycodes (not key-name strings), so we skip the Text
//! Services Manager entirely and map keycodes to rdev::Key ourselves.

use std::ffi::c_void;

use rdev::{EventType, Key};

// ---------------------------------------------------------------------------
// Core Foundation / Core Graphics FFI
// ---------------------------------------------------------------------------

type CFMachPortRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;
type CFRunLoopRef = *mut c_void;
type CFStringRef = *const c_void;
type CGEventRef = *mut c_void;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: unsafe extern "C" fn(*mut c_void, u32, CGEventRef, *mut c_void) -> CGEventRef,
        user_info: *mut c_void,
    ) -> CFMachPortRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventGetFlags(event: CGEventRef) -> u64;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: CFMachPortRef,
        order: i64,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    fn CFRunLoopRun();

    static kCFRunLoopCommonModes: CFStringRef;
}

// CGEventTapCreate constants
const K_CG_SESSION_EVENT_TAP: u32 = 1;
const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const K_CG_EVENT_TAP_OPTION_LISTEN_ONLY: u32 = 1;

// CGEventType values
const K_CG_EVENT_KEY_DOWN: u32 = 10;
const K_CG_EVENT_KEY_UP: u32 = 11;
const K_CG_EVENT_FLAGS_CHANGED: u32 = 12;
const K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFFFFFE;

// CGEventField for virtual keycode
const K_CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;

// CGEventFlags masks for modifier keys
const K_CG_EVENT_FLAG_MASK_ALPHA_SHIFT: u64 = 0x0001_0000; // Caps Lock
const K_CG_EVENT_FLAG_MASK_SHIFT: u64 = 0x0002_0000;
const K_CG_EVENT_FLAG_MASK_CONTROL: u64 = 0x0004_0000;
const K_CG_EVENT_FLAG_MASK_ALTERNATE: u64 = 0x0008_0000;
const K_CG_EVENT_FLAG_MASK_COMMAND: u64 = 0x0010_0000;

// ---------------------------------------------------------------------------
// Tap context — passed through CGEventTapCreate's userInfo pointer
// ---------------------------------------------------------------------------

struct TapContext<F> {
    callback: F,
    tap: CFMachPortRef,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Start a CGEvent tap that delivers `KeyPress` / `KeyRelease` events via
/// `callback`.  Blocks the calling thread (runs a CFRunLoop).
pub fn listen<F>(callback: F)
where
    F: FnMut(EventType) + 'static,
{
    let mask: u64 =
        (1 << K_CG_EVENT_KEY_DOWN) | (1 << K_CG_EVENT_KEY_UP) | (1 << K_CG_EVENT_FLAGS_CHANGED);

    let context = Box::into_raw(Box::new(TapContext {
        callback,
        tap: std::ptr::null_mut(),
    }));

    unsafe {
        let tap = CGEventTapCreate(
            K_CG_SESSION_EVENT_TAP,
            K_CG_HEAD_INSERT_EVENT_TAP,
            K_CG_EVENT_TAP_OPTION_LISTEN_ONLY,
            mask,
            raw_callback::<F>,
            context as *mut c_void,
        );

        if tap.is_null() {
            eprintln!("Failed to create CGEvent tap — is Accessibility permission granted?");
            let _ = Box::from_raw(context);
            return;
        }

        // Store the tap ref so the callback can re-enable it on timeout.
        (*context).tap = tap;

        let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
        let run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);

        CFRunLoopRun(); // blocks forever
    }
}

// ---------------------------------------------------------------------------
// CGEvent tap callback (C ABI)
// ---------------------------------------------------------------------------

unsafe extern "C" fn raw_callback<F: FnMut(EventType)>(
    _proxy: *mut c_void,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    let ctx = &mut *(user_info as *mut TapContext<F>);

    // macOS disables the tap after a timeout — re-enable it.
    if event_type == K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT {
        if !ctx.tap.is_null() {
            CGEventTapEnable(ctx.tap, true);
        }
        return event;
    }

    let keycode = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u16;

    match event_type {
        K_CG_EVENT_KEY_DOWN => {
            if let Some(key) = keycode_to_key(keycode) {
                (ctx.callback)(EventType::KeyPress(key));
            }
        }
        K_CG_EVENT_KEY_UP => {
            if let Some(key) = keycode_to_key(keycode) {
                (ctx.callback)(EventType::KeyRelease(key));
            }
        }
        K_CG_EVENT_FLAGS_CHANGED => {
            if let Some(key) = keycode_to_key(keycode) {
                let flags = CGEventGetFlags(event);
                if is_modifier_pressed(keycode, flags) {
                    (ctx.callback)(EventType::KeyPress(key));
                } else {
                    (ctx.callback)(EventType::KeyRelease(key));
                }
            }
        }
        _ => {}
    }

    event
}

// ---------------------------------------------------------------------------
// Modifier-flag helpers
// ---------------------------------------------------------------------------

fn is_modifier_pressed(keycode: u16, flags: u64) -> bool {
    let mask = match keycode {
        0x38 | 0x3C => K_CG_EVENT_FLAG_MASK_SHIFT,
        0x3B | 0x3E => K_CG_EVENT_FLAG_MASK_CONTROL,
        0x3A | 0x3D => K_CG_EVENT_FLAG_MASK_ALTERNATE,
        0x37 | 0x36 => K_CG_EVENT_FLAG_MASK_COMMAND,
        0x39 => K_CG_EVENT_FLAG_MASK_ALPHA_SHIFT,
        _ => return false,
    };
    flags & mask != 0
}

// ---------------------------------------------------------------------------
// macOS virtual-keycode → rdev::Key mapping
// ---------------------------------------------------------------------------

fn keycode_to_key(code: u16) -> Option<Key> {
    Some(match code {
        // Letters (ANSI layout order)
        0x00 => Key::KeyA,
        0x01 => Key::KeyS,
        0x02 => Key::KeyD,
        0x03 => Key::KeyF,
        0x04 => Key::KeyH,
        0x05 => Key::KeyG,
        0x06 => Key::KeyZ,
        0x07 => Key::KeyX,
        0x08 => Key::KeyC,
        0x09 => Key::KeyV,
        0x0B => Key::KeyB,
        0x0C => Key::KeyQ,
        0x0D => Key::KeyW,
        0x0E => Key::KeyE,
        0x0F => Key::KeyR,
        0x10 => Key::KeyY,
        0x11 => Key::KeyT,
        0x1F => Key::KeyO,
        0x20 => Key::KeyU,
        0x22 => Key::KeyI,
        0x23 => Key::KeyP,
        0x25 => Key::KeyL,
        0x26 => Key::KeyJ,
        0x28 => Key::KeyK,
        0x2D => Key::KeyN,
        0x2E => Key::KeyM,

        // Number row
        0x12 => Key::Num1,
        0x13 => Key::Num2,
        0x14 => Key::Num3,
        0x15 => Key::Num4,
        0x17 => Key::Num5,
        0x16 => Key::Num6,
        0x1A => Key::Num7,
        0x1C => Key::Num8,
        0x19 => Key::Num9,
        0x1D => Key::Num0,

        // Punctuation / symbols
        0x18 => Key::Equal,
        0x1B => Key::Minus,
        0x1E => Key::RightBracket,
        0x21 => Key::LeftBracket,
        0x27 => Key::Quote,
        0x29 => Key::SemiColon,
        0x2A => Key::BackSlash,
        0x2B => Key::Comma,
        0x2C => Key::Slash,
        0x2F => Key::Dot,
        0x32 => Key::BackQuote,

        // Special keys
        0x24 => Key::Return,
        0x30 => Key::Tab,
        0x31 => Key::Space,
        0x33 => Key::Backspace,
        0x35 => Key::Escape,
        0x39 => Key::CapsLock,
        0x75 => Key::Delete,   // Forward Delete
        0x73 => Key::Home,
        0x77 => Key::End,
        0x74 => Key::PageUp,
        0x79 => Key::PageDown,

        // Arrow keys
        0x7B => Key::LeftArrow,
        0x7C => Key::RightArrow,
        0x7D => Key::DownArrow,
        0x7E => Key::UpArrow,

        // Modifier keys
        0x37 => Key::MetaLeft,
        0x36 => Key::MetaRight,
        0x38 => Key::ShiftLeft,
        0x3C => Key::ShiftRight,
        0x3A => Key::Alt,
        0x3D => Key::AltGr,
        0x3B => Key::ControlLeft,
        0x3E => Key::ControlRight,

        // Function keys
        0x7A => Key::F1,
        0x78 => Key::F2,
        0x63 => Key::F3,
        0x76 => Key::F4,
        0x60 => Key::F5,
        0x61 => Key::F6,
        0x62 => Key::F7,
        0x64 => Key::F8,
        0x65 => Key::F9,
        0x6D => Key::F10,
        0x67 => Key::F11,
        0x6F => Key::F12,

        // Numpad
        0x52 => Key::Kp0,
        0x53 => Key::Kp1,
        0x54 => Key::Kp2,
        0x55 => Key::Kp3,
        0x56 => Key::Kp4,
        0x57 => Key::Kp5,
        0x58 => Key::Kp6,
        0x59 => Key::Kp7,
        0x5B => Key::Kp8,
        0x5C => Key::Kp9,
        0x41 => Key::KpDelete,
        0x43 => Key::KpMultiply,
        0x45 => Key::KpPlus,
        0x4B => Key::KpDivide,
        0x4C => Key::KpReturn,
        0x4E => Key::KpMinus,

        _ => return None,
    })
}
