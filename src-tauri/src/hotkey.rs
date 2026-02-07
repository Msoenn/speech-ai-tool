use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::error::AppError;
use crate::pipeline::{self, PipelineStatus, PipelineStatusEvent};
use crate::sounds;
use crate::tray;
use crate::AppState;

pub fn register_hotkey(app: &AppHandle, hotkey_str: &str) -> Result<(), AppError> {
    let shortcut: Shortcut = hotkey_str
        .parse()
        .map_err(|e| AppError::Hotkey(format!("Invalid hotkey '{}': {}", hotkey_str, e)))?;

    // Unregister all existing shortcuts first
    let gsm = app.global_shortcut();
    let _ = gsm.unregister_all();

    let app_handle = app.clone();
    gsm.on_shortcut(shortcut, move |_app, _shortcut, event| {
        match event.state {
            ShortcutState::Pressed => {
                let state = _app.state::<AppState>();
                let settings = state.settings.lock().unwrap();
                let device_index = settings.audio_device_index;
                drop(settings);

                // Play start tone, update tray, show overlay
                state.sound_player.play(sounds::START_TONE);
                tray::set_tray_status(_app, "recording");
                tray::show_overlay(_app);

                // Start recording
                if let Err(e) = state.recorder.lock().unwrap().start_recording(device_index) {
                    eprintln!("Failed to start recording: {}", e);
                    let _ = _app.emit(
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

                let _ = _app.emit(
                    "pipeline-status",
                    PipelineStatusEvent {
                        status: PipelineStatus::Recording,
                        raw_text: None,
                        cleaned_text: None,
                        error: None,
                    },
                );
            }
            ShortcutState::Released => {
                // Play stop tone
                let state = _app.state::<AppState>();
                state.sound_player.play(sounds::STOP_TONE);

                let app_clone = app_handle.clone();
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
        }
    })
    .map_err(|e| AppError::Hotkey(format!("Failed to register hotkey: {}", e)))?;

    Ok(())
}
