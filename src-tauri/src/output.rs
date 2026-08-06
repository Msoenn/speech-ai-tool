use crate::error::AppError;
use tauri_plugin_clipboard_manager::ClipboardExt;

/// A modifier in a paste shortcut, independent of any backend's key enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteModifier {
    Control,
    Shift,
    Alt,
    Meta,
}

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
