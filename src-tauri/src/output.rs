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
        if let Err(e) = simulate_paste(paste_shortcut) {
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

fn simulate_paste(paste_shortcut: &str) -> Result<(), AppError> {
    let (modifiers, char_key) = parse_paste_shortcut(paste_shortcut)?;

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| AppError::Output(format!("Failed to create enigo: {}", e)))?;

    // Defensively release all common modifiers to ensure clean state
    let all_modifiers = [Key::Control, Key::Shift, Key::Alt, Key::Meta];
    for m in &all_modifiers {
        let _ = enigo.key(*m, Direction::Release);
    }
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Press modifiers
    for m in &modifiers {
        enigo
            .key(*m, Direction::Press)
            .map_err(|e| AppError::Output(format!("Key press failed: {}", e)))?;
    }

    // Press the key
    enigo
        .key(char_key, Direction::Click)
        .map_err(|e| AppError::Output(format!("Key click failed: {}", e)))?;

    // Release modifiers in reverse order
    for m in modifiers.iter().rev() {
        enigo
            .key(*m, Direction::Release)
            .map_err(|e| AppError::Output(format!("Key release failed: {}", e)))?;
    }

    Ok(())
}
