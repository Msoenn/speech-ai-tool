use crate::error::AppError;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use tauri_plugin_clipboard_manager::ClipboardExt;

pub fn copy_to_clipboard(app: &tauri::AppHandle, text: &str) -> Result<(), AppError> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| AppError::Output(format!("Clipboard write failed: {}", e)))?;
    Ok(())
}

pub fn copy_and_paste(
    app: &tauri::AppHandle,
    text: &str,
    auto_paste: bool,
    paste_shortcut: &str,
) -> Result<(), AppError> {
    // Write to clipboard
    app.clipboard()
        .write_text(text)
        .map_err(|e| AppError::Output(format!("Clipboard write failed: {}", e)))?;

    if auto_paste {
        // Small delay to ensure clipboard is ready
        std::thread::sleep(std::time::Duration::from_millis(100));
        if let Err(e) = simulate_paste(app, paste_shortcut) {
            eprintln!("Auto-paste failed (text is in clipboard): {}", e);
        }
    }

    Ok(())
}

/// Parse a shortcut string like "Ctrl+Shift+V" or "Cmd+V" into modifier keys + a character.
fn parse_paste_shortcut(shortcut: &str) -> Result<(Vec<Key>, Key), AppError> {
    let parts: Vec<&str> = shortcut.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return Err(AppError::Output("Empty paste shortcut".into()));
    }

    let mut modifiers = Vec::new();
    for part in &parts[..parts.len() - 1] {
        let key = match part.to_lowercase().as_str() {
            "ctrl" | "control" => Key::Control,
            "shift" => Key::Shift,
            "alt" => Key::Alt,
            "cmd" | "meta" | "super" => Key::Meta,
            other => {
                return Err(AppError::Output(format!(
                    "Unknown modifier in paste shortcut: {}",
                    other
                )))
            }
        };
        modifiers.push(key);
    }

    let last = parts.last().unwrap();
    let char_key = if last.len() == 1 {
        Key::Unicode(last.to_lowercase().chars().next().unwrap())
    } else {
        return Err(AppError::Output(format!(
            "Invalid key in paste shortcut: {}",
            last
        )));
    };

    Ok((modifiers, char_key))
}

fn simulate_paste(app: &tauri::AppHandle, paste_shortcut: &str) -> Result<(), AppError> {
    let (modifiers, char_key) = parse_paste_shortcut(paste_shortcut)?;

    // On macOS 26.3+, enigo's character-key path calls TSMGetInputSourceProperty
    // (Text Services Manager) to resolve the layout-dependent keycode. TSM
    // hard-asserts it is called on the main dispatch queue and SIGTRAPs on any
    // other thread. copy_and_paste runs on a tokio worker, so the synthesis must
    // be marshalled onto the main thread. (This is the same TSM-on-a-background-
    // thread crash that macos_event_tap.rs works around for key *listening*.)
    #[cfg(target_os = "macos")]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        app.run_on_main_thread(move || {
            let _ = tx.send(press_paste_chord(&modifiers, char_key));
        })
        .map_err(|e| {
            AppError::Output(format!("Failed to dispatch paste to main thread: {}", e))
        })?;
        rx.recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|e| AppError::Output(format!("Paste task did not complete: {}", e)))?
            .map_err(AppError::Output)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        press_paste_chord(&modifiers, char_key).map_err(AppError::Output)
    }
}

/// Synthesize the paste chord (modifiers + key). On macOS this MUST run on the
/// main thread — see the note in `simulate_paste`.
fn press_paste_chord(modifiers: &[Key], char_key: Key) -> Result<(), String> {
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("Failed to create enigo: {}", e))?;

    // Defensively release all common modifiers to ensure clean state (e.g. the
    // hotkey's own modifiers may still be physically held).
    let all_modifiers = [Key::Control, Key::Shift, Key::Alt, Key::Meta];
    for m in &all_modifiers {
        let _ = enigo.key(*m, Direction::Release);
    }
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Press modifiers
    for m in modifiers {
        enigo
            .key(*m, Direction::Press)
            .map_err(|e| format!("Key press failed: {}", e))?;
    }

    // Press the key
    enigo
        .key(char_key, Direction::Click)
        .map_err(|e| format!("Key click failed: {}", e))?;

    // Release modifiers in reverse order
    for m in modifiers.iter().rev() {
        enigo
            .key(*m, Direction::Release)
            .map_err(|e| format!("Key release failed: {}", e))?;
    }

    Ok(())
}
