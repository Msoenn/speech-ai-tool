mod audio;
mod error;
mod history;
mod hotkey;
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

#[tauri::command]
fn list_audio_devices() -> Result<Vec<audio::AudioDevice>, AppError> {
    audio::list_input_devices()
}

#[tauri::command]
fn start_recording(
    state: tauri::State<'_, AppState>,
    device_index: Option<usize>,
) -> Result<(), AppError> {
    state.recorder.lock().unwrap().start_recording(device_index)
}

#[tauri::command]
fn stop_recording(state: tauri::State<'_, AppState>) -> Result<Vec<u8>, AppError> {
    state.recorder.lock().unwrap().stop_recording()
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
fn load_whisper_model(
    state: tauri::State<'_, AppState>,
    model_name: String,
) -> Result<(), AppError> {
    state.whisper.load_model(&model_name)
}

#[tauri::command]
fn transcribe_audio(
    state: tauri::State<'_, AppState>,
    wav_bytes: Vec<u8>,
) -> Result<String, AppError> {
    state.whisper.transcribe(&wav_bytes)
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

            // Start the rdev hotkey listener
            let hotkey_state = hotkey::start_listener(app.handle(), &loaded_settings.hotkey);

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
