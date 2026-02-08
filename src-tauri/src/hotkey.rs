use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use rdev::{self, EventType, Key};
use tauri::{AppHandle, Emitter, Manager};

use crate::pipeline::{self, PipelineStatus, PipelineStatusEvent};
use crate::sounds;
use crate::tray;
use crate::AppState;

pub struct HotkeyState {
    /// The keys that make up the configured hotkey combo
    combo: Mutex<Vec<Key>>,
    /// Whether the combo is currently held down (recording active)
    combo_active: AtomicBool,
    /// Paused during UI hotkey recording to prevent conflicts
    paused: AtomicBool,
}

impl HotkeyState {
    pub fn new(combo: Vec<Key>) -> Self {
        Self {
            combo: Mutex::new(combo),
            combo_active: AtomicBool::new(false),
            paused: AtomicBool::new(false),
        }
    }
}

/// Start the rdev global listener thread. Returns shared state for runtime updates.
pub fn start_listener(app: &AppHandle, hotkey_str: &str) -> Arc<HotkeyState> {
    let combo = parse_hotkey_string(hotkey_str);
    let state = Arc::new(HotkeyState::new(combo));
    let state_clone = Arc::clone(&state);
    let app_handle = app.clone();

    thread::spawn(move || {
        let mut held_keys: HashSet<Key> = HashSet::new();

        let callback = move |event: rdev::Event| {
            match event.event_type {
                EventType::KeyPress(key) => {
                    held_keys.insert(key);
                    check_combo(&held_keys, &state_clone, &app_handle);
                }
                EventType::KeyRelease(key) => {
                    held_keys.remove(&key);
                    check_combo(&held_keys, &state_clone, &app_handle);
                }
                _ => {}
            }
        };

        if let Err(e) = rdev::listen(callback) {
            eprintln!("rdev listener error: {:?}", e);
        }
    });

    state
}

fn check_combo(held_keys: &HashSet<Key>, state: &Arc<HotkeyState>, app: &AppHandle) {
    if state.paused.load(Ordering::Relaxed) {
        return;
    }

    let combo = state.combo.lock().unwrap();
    if combo.is_empty() {
        return;
    }

    let all_held = combo.iter().all(|k| held_keys.contains(k));
    drop(combo);

    let was_active = state.combo_active.load(Ordering::Relaxed);

    if all_held && !was_active {
        state.combo_active.store(true, Ordering::Relaxed);
        on_hotkey_pressed(app);
    } else if !all_held && was_active {
        state.combo_active.store(false, Ordering::Relaxed);
        on_hotkey_released(app);
    }
}

fn on_hotkey_pressed(app: &AppHandle) {
    let app_state = app.state::<AppState>();
    let settings = app_state.settings.lock().unwrap();
    let device_index = settings.audio_device_index;
    drop(settings);

    app_state.sound_player.play(sounds::START_TONE);
    tray::set_tray_status(app, "recording");
    tray::show_overlay(app);

    if let Err(e) = app_state
        .recorder
        .lock()
        .unwrap()
        .start_recording(device_index)
    {
        eprintln!("Failed to start recording: {}", e);
        let _ = app.emit(
            "pipeline-status",
            PipelineStatusEvent {
                status: PipelineStatus::Error,
                raw_text: None,
                cleaned_text: None,
                error: Some(format!("Failed to start recording: {}", e)),
            },
        );
        return;
    }

    let _ = app.emit(
        "pipeline-status",
        PipelineStatusEvent {
            status: PipelineStatus::Recording,
            raw_text: None,
            cleaned_text: None,
            error: None,
        },
    );
}

fn on_hotkey_released(app: &AppHandle) {
    let app_state = app.state::<AppState>();
    app_state.sound_player.play(sounds::STOP_TONE);

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = pipeline::run_pipeline(app_clone.clone()).await {
            eprintln!("Pipeline error: {}", e);
            tray::set_tray_status(&app_clone, "idle");
            tray::hide_overlay(&app_clone);
            let _ = app_clone.emit(
                "pipeline-status",
                PipelineStatusEvent {
                    status: PipelineStatus::Error,
                    raw_text: None,
                    cleaned_text: None,
                    error: Some(e.to_string()),
                },
            );
        }
    });
}

/// Update the hotkey combo at runtime (no need to re-register).
pub fn update_hotkey(state: &Arc<HotkeyState>, hotkey_str: &str) {
    let new_combo = parse_hotkey_string(hotkey_str);
    *state.combo.lock().unwrap() = new_combo;
    state.combo_active.store(false, Ordering::Relaxed);
}

/// Pause/unpause the listener (used during UI hotkey recording).
pub fn set_paused(state: &Arc<HotkeyState>, paused: bool) {
    state.paused.store(paused, Ordering::Relaxed);
    if paused {
        state.combo_active.store(false, Ordering::Relaxed);
    }
}

/// Parse a hotkey string like "ControlLeft+ShiftLeft+Space" into rdev keys.
pub fn parse_hotkey_string(hotkey_str: &str) -> Vec<Key> {
    hotkey_str
        .split('+')
        .filter(|s| !s.is_empty())
        .filter_map(|s| parse_key_name(s.trim()))
        .collect()
}

/// Map a key name string to an rdev::Key variant.
fn parse_key_name(name: &str) -> Option<Key> {
    Some(match name {
        // Modifiers — left
        "ControlLeft" => Key::ControlLeft,
        "ShiftLeft" => Key::ShiftLeft,
        "AltLeft" | "Alt" => Key::Alt,
        "MetaLeft" | "SuperLeft" => Key::MetaLeft,
        // Modifiers — right
        "ControlRight" => Key::ControlRight,
        "ShiftRight" => Key::ShiftRight,
        "AltRight" | "AltGr" => Key::AltGr,
        "MetaRight" | "SuperRight" => Key::MetaRight,
        // Common keys
        "Space" => Key::Space,
        "Enter" | "Return" => Key::Return,
        "Tab" => Key::Tab,
        "Escape" | "Esc" => Key::Escape,
        "Backspace" => Key::Backspace,
        "Delete" => Key::Delete,
        "Insert" => Key::Insert,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        "CapsLock" => Key::CapsLock,
        // Arrow keys
        "ArrowUp" | "Up" => Key::UpArrow,
        "ArrowDown" | "Down" => Key::DownArrow,
        "ArrowLeft" | "Left" => Key::LeftArrow,
        "ArrowRight" | "Right" => Key::RightArrow,
        // Function keys
        "F1" => Key::F1,
        "F2" => Key::F2,
        "F3" => Key::F3,
        "F4" => Key::F4,
        "F5" => Key::F5,
        "F6" => Key::F6,
        "F7" => Key::F7,
        "F8" => Key::F8,
        "F9" => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,
        // Number keys
        "Digit0" | "0" => Key::Num0,
        "Digit1" | "1" => Key::Num1,
        "Digit2" | "2" => Key::Num2,
        "Digit3" | "3" => Key::Num3,
        "Digit4" | "4" => Key::Num4,
        "Digit5" | "5" => Key::Num5,
        "Digit6" | "6" => Key::Num6,
        "Digit7" | "7" => Key::Num7,
        "Digit8" | "8" => Key::Num8,
        "Digit9" | "9" => Key::Num9,
        // Letter keys
        "KeyA" | "A" => Key::KeyA,
        "KeyB" | "B" => Key::KeyB,
        "KeyC" | "C" => Key::KeyC,
        "KeyD" | "D" => Key::KeyD,
        "KeyE" | "E" => Key::KeyE,
        "KeyF" | "F" => Key::KeyF,
        "KeyG" | "G" => Key::KeyG,
        "KeyH" | "H" => Key::KeyH,
        "KeyI" | "I" => Key::KeyI,
        "KeyJ" | "J" => Key::KeyJ,
        "KeyK" | "K" => Key::KeyK,
        "KeyL" | "L" => Key::KeyL,
        "KeyM" | "M" => Key::KeyM,
        "KeyN" | "N" => Key::KeyN,
        "KeyO" | "O" => Key::KeyO,
        "KeyP" | "P" => Key::KeyP,
        "KeyQ" | "Q" => Key::KeyQ,
        "KeyR" | "R" => Key::KeyR,
        "KeyS" | "S" => Key::KeyS,
        "KeyT" | "T" => Key::KeyT,
        "KeyU" | "U" => Key::KeyU,
        "KeyV" | "V" => Key::KeyV,
        "KeyW" | "W" => Key::KeyW,
        "KeyX" | "X" => Key::KeyX,
        "KeyY" | "Y" => Key::KeyY,
        "KeyZ" | "Z" => Key::KeyZ,
        // Punctuation / symbols
        "Minus" => Key::Minus,
        "Equal" => Key::Equal,
        "BracketLeft" => Key::LeftBracket,
        "BracketRight" => Key::RightBracket,
        "Backslash" => Key::BackSlash,
        "Semicolon" => Key::SemiColon,
        "Quote" => Key::Quote,
        "Backquote" => Key::BackQuote,
        "Comma" => Key::Comma,
        "Period" => Key::Dot,
        "Slash" => Key::Slash,
        // Numpad
        "Numpad0" => Key::Kp0,
        "Numpad1" => Key::Kp1,
        "Numpad2" => Key::Kp2,
        "Numpad3" => Key::Kp3,
        "Numpad4" => Key::Kp4,
        "Numpad5" => Key::Kp5,
        "Numpad6" => Key::Kp6,
        "Numpad7" => Key::Kp7,
        "Numpad8" => Key::Kp8,
        "Numpad9" => Key::Kp9,
        "NumpadDecimal" => Key::KpDelete,
        "NumpadAdd" => Key::KpPlus,
        "NumpadSubtract" => Key::KpMinus,
        "NumpadMultiply" => Key::KpMultiply,
        "NumpadDivide" => Key::KpDivide,
        "NumpadEnter" => Key::KpReturn,
        _ => {
            eprintln!("Unknown key name in hotkey string: {}", name);
            return None;
        }
    })
}

/// Convert old-format hotkey strings to the new rdev-compatible format.
/// e.g. "CmdOrCtrl+Shift+Space" → "ControlLeft+ShiftLeft+Space"
pub fn migrate_hotkey_format(hotkey: &str) -> String {
    let parts: Vec<&str> = hotkey.split('+').collect();
    let mut new_parts: Vec<&str> = Vec::new();

    for part in &parts {
        let mapped = match *part {
            "CmdOrCtrl" | "CommandOrControl" | "Ctrl" | "Control" => {
                if cfg!(target_os = "macos") {
                    "MetaLeft"
                } else {
                    "ControlLeft"
                }
            }
            "Cmd" | "Command" | "Meta" | "Super" => "MetaLeft",
            "Shift" => "ShiftLeft",
            "Alt" => "AltLeft",
            "AltGr" => "AltRight",
            other => other,
        };
        new_parts.push(mapped);
    }

    new_parts.join("+")
}

/// Check if a hotkey string uses the old format and needs migration.
pub fn needs_migration(hotkey: &str) -> bool {
    let old_names = [
        "CmdOrCtrl",
        "CommandOrControl",
        "Ctrl",
        "Control",
        "Cmd",
        "Command",
        "Meta",
        "Super",
        "Shift",
        "Alt",
    ];
    hotkey.split('+').any(|part| old_names.contains(&part))
}
