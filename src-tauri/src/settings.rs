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
    #[serde(default = "default_whisper_api_model")]
    pub whisper_api_model: String,
    pub llm: LlmConfig,
    pub auto_paste: bool,
    #[serde(default = "default_paste_shortcut")]
    pub paste_shortcut: String,
    pub history_max_items: usize,
    /// Color theme id (see src/lib/palettes.ts). Frontend-only preference.
    #[serde(default = "default_palette")]
    pub palette: String,
    /// Whether to show the recording status overlay window.
    #[serde(default = "default_show_overlay")]
    pub show_overlay: bool,
}

/// True on Linux under a Wayland session. GNOME Wayland forces focus onto
/// newly mapped windows, so the overlay would steal focus; default it off there.
/// Detected via XDG_SESSION_TYPE or, more robustly, a set WAYLAND_DISPLAY
/// (present even for nested/container Wayland where the session var may be absent).
pub fn is_wayland_session() -> bool {
    cfg!(target_os = "linux")
        && (std::env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland")
            || std::env::var("WAYLAND_DISPLAY").map(|v| !v.is_empty()).unwrap_or(false))
}

pub fn default_show_overlay() -> bool {
    !is_wayland_session()
}

pub fn default_palette() -> String {
    "slate".to_string()
}

pub fn default_whisper_language() -> String {
    "en".to_string()
}

pub fn default_whisper_api_model() -> String {
    "whisper-1".to_string()
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
            whisper_api_model: default_whisper_api_model(),
            llm: LlmConfig::default(),
            auto_paste: true,
            paste_shortcut: default_paste_shortcut(),
            history_max_items: 100,
            palette: default_palette(),
            show_overlay: default_show_overlay(),
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

    // Auto-upgrade the cleanup prompt and few-shot examples for anyone still on
    // a shipped default (i.e. never customized): move them to the current
    // defaults. Customized values match no known default and are left untouched.
    if let Some(prompt) = crate::llm::upgraded_default_prompt(&settings.llm.system_prompt) {
        settings.llm.system_prompt = prompt.to_string();
    }
    if let Some(examples) = crate::llm::upgraded_default_few_shot(&settings.llm.few_shot_examples) {
        settings.llm.few_shot_examples = examples;
    }

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
