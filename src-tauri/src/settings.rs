use serde::{Deserialize, Serialize};

use crate::llm::LlmConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub audio_device_index: Option<usize>,
    pub hotkey: String,
    pub whisper_mode: WhisperMode,
    pub whisper_model: String,
    #[serde(default = "default_whisper_language")]
    pub whisper_language: String,
    pub whisper_api_endpoint: String,
    pub whisper_api_key: String,
    pub llm: LlmConfig,
    pub auto_paste: bool,
    #[serde(default = "default_paste_shortcut")]
    pub paste_shortcut: String,
    pub history_max_items: usize,
}

pub fn default_whisper_language() -> String {
    "en".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WhisperMode {
    Local,
    Api,
}

pub fn default_paste_shortcut() -> String {
    if cfg!(target_os = "macos") {
        "Cmd+V".to_string()
    } else if cfg!(target_os = "linux") {
        "Ctrl+Shift+V".to_string()
    } else {
        "Ctrl+V".to_string()
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            audio_device_index: None,
            hotkey: if cfg!(target_os = "macos") {
                "MetaLeft+ShiftLeft+Space".to_string()
            } else {
                "ControlLeft+ShiftLeft+Space".to_string()
            },
            whisper_mode: WhisperMode::Local,
            whisper_model: "large-v3-turbo-q5_0".to_string(),
            whisper_language: default_whisper_language(),
            whisper_api_endpoint: String::new(),
            whisper_api_key: String::new(),
            llm: LlmConfig::default(),
            auto_paste: true,
            paste_shortcut: default_paste_shortcut(),
            history_max_items: 100,
        }
    }
}

pub fn load_settings(store: &tauri_plugin_store::Store<tauri::Wry>) -> AppSettings {
    let mut settings: AppSettings = store
        .get("settings")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    // Migrate old hotkey format (e.g. "CmdOrCtrl+Shift+Space") to new format
    if crate::hotkey::needs_migration(&settings.hotkey) {
        settings.hotkey = crate::hotkey::migrate_hotkey_format(&settings.hotkey);
    }

    // Migrate old model names to curated quantized variants
    settings.whisper_model = match settings.whisper_model.as_str() {
        "tiny" | "base" => "tiny-q5_1".to_string(),
        "small" => "small-q5_1".to_string(),
        "medium" => "large-v3-turbo-q5_0".to_string(),
        _ => settings.whisper_model,
    };

    settings
}

pub fn save_settings(
    store: &tauri_plugin_store::Store<tauri::Wry>,
    settings: &AppSettings,
) -> Result<(), crate::error::AppError> {
    let value = serde_json::to_value(settings)
        .map_err(|e| crate::error::AppError::Settings(e.to_string()))?;
    store.set("settings", value);
    store
        .save()
        .map_err(|e| crate::error::AppError::Settings(e.to_string()))?;
    Ok(())
}
