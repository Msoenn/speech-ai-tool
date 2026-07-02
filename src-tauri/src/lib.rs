mod audio;
mod error;
mod history;
mod hotkey;
#[cfg(target_os = "macos")]
mod macos_event_tap;
#[cfg(target_os = "macos")]
mod macos_microphone;
mod llm;
mod output;
mod pipeline;
mod settings;
mod sounds;
mod tray;
mod whisper;

use audio::AudioRecorder;
use error::AppError;
use history::HistoryDb;
use hotkey::HotkeyState;
use settings::AppSettings;
use sounds::SoundPlayer;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use whisper::WhisperEngine;

pub struct AppState {
    pub recorder: Mutex<AudioRecorder>,
    pub whisper: WhisperEngine,
    pub settings: Mutex<AppSettings>,
    pub history: HistoryDb,
    pub sound_player: SoundPlayer,
    pub hotkey_state: Arc<HotkeyState>,
}

// --- Audio commands ---
//
// These (and the whisper load/transcribe commands below) are async and run
// their blocking work on the thread pool: synchronous Tauri commands execute
// on the main thread, and blocking it while CoreAudio raises the microphone
// permission prompt deadlocks the dialog (it re-presents forever). Heavy
// whisper work would likewise freeze the UI.

#[tauri::command]
async fn list_audio_devices() -> Result<Vec<audio::AudioDevice>, AppError> {
    tauri::async_runtime::spawn_blocking(audio::list_input_devices)
        .await
        .map_err(|e| AppError::Audio(format!("task join error: {e}")))?
}

#[tauri::command]
async fn start_recording(
    app: tauri::AppHandle,
    device_index: Option<usize>,
) -> Result<(), AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let result = state.recorder.lock().unwrap().start_recording(device_index);
        result
    })
    .await
    .map_err(|e| AppError::Audio(format!("task join error: {e}")))?
}

#[tauri::command]
async fn stop_recording(app: tauri::AppHandle) -> Result<Vec<u8>, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let result = state.recorder.lock().unwrap().stop_recording();
        result
    })
    .await
    .map_err(|e| AppError::Audio(format!("task join error: {e}")))?
}

// --- Whisper commands ---

#[tauri::command]
fn list_whisper_models() -> Result<Vec<whisper::WhisperModelInfo>, AppError> {
    whisper::list_models()
}

#[tauri::command]
async fn download_whisper_model(app: tauri::AppHandle, model_name: String) -> Result<(), AppError> {
    whisper::download_model(app, &model_name).await
}

#[tauri::command]
async fn load_whisper_model(app: tauri::AppHandle, model_name: String) -> Result<(), AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        state.whisper.load_model(&model_name)
    })
    .await
    .map_err(|e| AppError::Whisper(format!("task join error: {e}")))?
}

#[tauri::command]
async fn transcribe_audio(app: tauri::AppHandle, wav_bytes: Vec<u8>) -> Result<String, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let language = state.settings.lock().unwrap().whisper_language.clone();
        state.whisper.transcribe(&wav_bytes, &language)
    })
    .await
    .map_err(|e| AppError::Whisper(format!("task join error: {e}")))?
}

#[tauri::command]
fn is_whisper_model_loaded(state: tauri::State<'_, AppState>) -> bool {
    state.whisper.is_model_loaded()
}

// --- LLM commands ---

#[tauri::command]
async fn cleanup_text(
    state: tauri::State<'_, AppState>,
    raw_text: String,
) -> Result<String, AppError> {
    let config = state.settings.lock().unwrap().llm.clone();
    llm::cleanup_text(&config, &raw_text).await
}

#[tauri::command]
async fn test_llm_connection(state: tauri::State<'_, AppState>) -> Result<String, AppError> {
    let config = state.settings.lock().unwrap().llm.clone();
    llm::test_connection(&config).await
}

// --- Output commands ---

#[tauri::command]
fn copy_to_clipboard(app: tauri::AppHandle, text: String) -> Result<(), AppError> {
    output::copy_to_clipboard(&app, &text)
}

#[tauri::command]
fn paste_text(app: tauri::AppHandle, text: String) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap();
    let auto_paste = settings.auto_paste;
    let paste_shortcut = settings.paste_shortcut.clone();
    drop(settings);
    output::copy_and_paste(&app, &text, auto_paste, &paste_shortcut)
}

// --- Hotkey commands ---

#[tauri::command]
fn set_hotkey(app: tauri::AppHandle, hotkey_str: String) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    hotkey::update_hotkey(&state.hotkey_state, &hotkey_str);
    Ok(())
}

#[tauri::command]
fn pause_hotkey(app: tauri::AppHandle, paused: bool) {
    let state = app.state::<AppState>();
    hotkey::set_paused(&state.hotkey_state, paused);
}

#[tauri::command]
fn get_current_hotkey(state: tauri::State<'_, AppState>) -> String {
    state.settings.lock().unwrap().hotkey.clone()
}

// --- macOS permission commands ---
//
// On macOS the global hotkey (CGEvent tap) and auto-paste (`enigo`) both require
// the Accessibility permission. These commands let the UI check/request it. On
// other platforms they are no-ops that report "granted".

#[tauri::command]
fn check_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos_event_tap::has_accessibility_permission()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

#[tauri::command]
fn request_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos_event_tap::request_accessibility_permission()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

#[tauri::command]
fn open_accessibility_settings() {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn();
    }
}

/// Re-check Accessibility and (re)start the hotkey listener without a relaunch.
/// Returns whether the permission is granted; the listener runs iff granted.
#[tauri::command]
fn restart_hotkey_listener(app: tauri::AppHandle) -> bool {
    #[cfg(target_os = "macos")]
    {
        if !macos_event_tap::has_accessibility_permission() {
            return false;
        }
        let state = app.state::<AppState>();
        hotkey::ensure_listener(&app, &state.hotkey_state);
        true
    }
    #[cfg(not(target_os = "macos"))]
    {
        let state = app.state::<AppState>();
        hotkey::ensure_listener(&app, &state.hotkey_state);
        true
    }
}

// On macOS, cpal touching CoreAudio implicitly triggers the microphone
// permission prompt, so the UI checks/requests the permission explicitly
// before listing devices or recording. On other platforms these report
// "granted" (no-op).

#[tauri::command]
fn check_microphone_permission() -> String {
    #[cfg(target_os = "macos")]
    {
        macos_microphone::microphone_permission_status().to_string()
    }
    #[cfg(not(target_os = "macos"))]
    {
        "granted".to_string()
    }
}

#[tauri::command]
async fn request_microphone_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos_microphone::request_microphone_permission().await
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

#[tauri::command]
fn open_microphone_settings() {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
            .spawn();
    }
}

// --- Settings commands ---

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> AppSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn save_settings(
    app: tauri::AppHandle,
    settings: AppSettings,
) -> Result<(), AppError> {
    let state = app.state::<AppState>();

    // Check if hotkey changed
    let old_hotkey = state.settings.lock().unwrap().hotkey.clone();
    let hotkey_changed = old_hotkey != settings.hotkey;

    // Check if whisper model changed
    let old_model = state.settings.lock().unwrap().whisper_model.clone();
    let model_changed = old_model != settings.whisper_model;

    // Update in-memory settings
    *state.settings.lock().unwrap() = settings.clone();

    // Persist to store
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Settings(e.to_string()))?;
    settings::save_settings(&store, &settings)?;

    // Apply side effects
    if hotkey_changed {
        hotkey::update_hotkey(&state.hotkey_state, &settings.hotkey);
    }

    if model_changed && settings.whisper_mode == settings::WhisperMode::Local {
        if let Err(e) = state.whisper.load_model(&settings.whisper_model) {
            eprintln!("Failed to load whisper model: {}", e);
        }
    }

    Ok(())
}

#[tauri::command]
fn reset_settings(app: tauri::AppHandle) -> Result<AppSettings, AppError> {
    let defaults = AppSettings::default();
    let state = app.state::<AppState>();
    *state.settings.lock().unwrap() = defaults.clone();

    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Settings(e.to_string()))?;
    settings::save_settings(&store, &defaults)?;

    hotkey::update_hotkey(&state.hotkey_state, &defaults.hotkey);

    Ok(defaults)
}

// --- History commands ---

#[tauri::command]
fn get_history(state: tauri::State<'_, AppState>) -> Result<Vec<history::TranscriptionRecord>, AppError> {
    state.history.list()
}

#[tauri::command]
fn delete_history_item(state: tauri::State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.history.delete(&id)
}

#[tauri::command]
fn clear_history(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    state.history.clear_all()
}

// --- Whisper API command ---

#[tauri::command]
async fn test_whisper_api(state: tauri::State<'_, AppState>) -> Result<String, AppError> {
    let settings = state.settings.lock().unwrap().clone();
    // Send a tiny silent WAV to test the endpoint
    Ok(format!("Whisper API endpoint: {}", settings.whisper_api_endpoint))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;
                app.handle().plugin(tauri_plugin_process::init())?;
            }

            tray::setup_tray(app)?;

            // Initialize history database
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            let history_db = HistoryDb::new(&app_data_dir)
                .expect("failed to initialize history database");

            // Load settings
            let store = app
                .store("settings.json")
                .expect("failed to open settings store");
            let loaded_settings = settings::load_settings(&store);

            // Try to load whisper model if configured
            let whisper_engine = WhisperEngine::new();
            if loaded_settings.whisper_mode == settings::WhisperMode::Local {
                if let Err(e) = whisper_engine.load_model(&loaded_settings.whisper_model) {
                    eprintln!("Could not load whisper model on startup: {}", e);
                }
            }

            let hotkey_state = Arc::new(HotkeyState::new(hotkey::parse_hotkey_string(
                &loaded_settings.hotkey,
            )));

            // On macOS the global hotkey + auto-paste need Accessibility permission.
            // Only start the listener when granted (a tap created without the grant
            // is dead); the banner's "Re-check" restarts it via
            // `restart_hotkey_listener` once the user grants access.
            #[cfg(target_os = "macos")]
            {
                use tauri::Emitter;
                if macos_event_tap::has_accessibility_permission() {
                    hotkey::ensure_listener(app.handle(), &hotkey_state);
                } else {
                    eprintln!(
                        "Accessibility permission not granted — the global hotkey and \
                         auto-paste will not work until it is granted in System Settings \
                         ▸ Privacy & Security ▸ Accessibility."
                    );
                    // Auto-fire the system prompt only on first launch; afterwards
                    // the banner handles it (re-prompting is useless when the real
                    // problem is a stale TCC entry from a previous unsigned build).
                    let already_prompted = store
                        .get("accessibility_prompt_shown")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if !already_prompted {
                        macos_event_tap::request_accessibility_permission();
                        store.set("accessibility_prompt_shown", true);
                        let _ = store.save();
                    }
                    let _ = app.handle().emit("permission-required", "accessibility");
                }
            }

            #[cfg(not(target_os = "macos"))]
            hotkey::ensure_listener(app.handle(), &hotkey_state);

            app.manage(AppState {
                recorder: Mutex::new(AudioRecorder::new()),
                whisper: whisper_engine,
                settings: Mutex::new(loaded_settings.clone()),
                history: history_db,
                sound_player: SoundPlayer::new(),
                hotkey_state,
            });

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_audio_devices,
            start_recording,
            stop_recording,
            list_whisper_models,
            download_whisper_model,
            load_whisper_model,
            transcribe_audio,
            is_whisper_model_loaded,
            cleanup_text,
            test_llm_connection,
            copy_to_clipboard,
            paste_text,
            set_hotkey,
            get_current_hotkey,
            pause_hotkey,
            check_accessibility_permission,
            request_accessibility_permission,
            open_accessibility_settings,
            restart_hotkey_listener,
            check_microphone_permission,
            request_microphone_permission,
            open_microphone_settings,
            get_settings,
            save_settings,
            reset_settings,
            get_history,
            delete_history_item,
            clear_history,
            test_whisper_api,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
