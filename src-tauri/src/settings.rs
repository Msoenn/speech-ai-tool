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
    /// API mode only: send the glossary in the `hotwords` field (Speaches and
    /// other faster-whisper servers) and not inside the prompt.
    #[serde(default)]
    pub whisper_api_hotwords: bool,
    /// Style example for the Whisper prompt (see whisper::build_initial_prompt).
    #[serde(default = "crate::whisper::default_style_example")]
    pub whisper_style_example: String,
    /// Glossary for the Whisper prompt: comma-separated words to spell right.
    #[serde(default = "crate::whisper::default_glossary")]
    pub whisper_glossary: String,
    pub llm: LlmConfig,
    /// Whether the LLM cleans up the transcription. If false, the raw Whisper
    /// text is pasted.
    #[serde(default = "default_llm_cleanup_enabled")]
    pub llm_cleanup_enabled: bool,
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

pub fn default_llm_cleanup_enabled() -> bool {
    true
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

impl AppSettings {
    /// The prompt sent to Whisper, or `None` if both prompt boxes are empty.
    pub fn whisper_initial_prompt(&self) -> Option<String> {
        crate::whisper::build_initial_prompt(&self.whisper_style_example, &self.whisper_glossary)
    }
}

impl AppSettings {
    /// What API mode sends, as (prompt, hotwords).
    ///
    /// By default the glossary is part of the prompt. If the user turned on
    /// hotwords, the prompt holds only the style example and the glossary
    /// goes in the `hotwords` field, which a faster-whisper server applies to
    /// the whole recording.
    pub fn whisper_api_prompt_parts(&self) -> (Option<String>, Option<String>) {
        if self.whisper_api_hotwords {
            (
                crate::whisper::build_initial_prompt(&self.whisper_style_example, ""),
                crate::whisper::glossary_terms(&self.whisper_glossary),
            )
        } else {
            (self.whisper_initial_prompt(), None)
        }
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
            whisper_api_hotwords: false,
            whisper_style_example: crate::whisper::default_style_example(),
            whisper_glossary: crate::whisper::default_glossary(),
            llm: LlmConfig::default(),
            llm_cleanup_enabled: default_llm_cleanup_enabled(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_saved_before_these_fields_get_the_defaults() {
        let mut value = serde_json::to_value(AppSettings::default()).unwrap();
        let obj = value.as_object_mut().unwrap();
        obj.remove("llm_cleanup_enabled");
        obj.remove("whisper_style_example");
        obj.remove("whisper_glossary");

        let settings: AppSettings = serde_json::from_value(value).unwrap();
        assert!(settings.llm_cleanup_enabled);
        assert_eq!(settings.whisper_glossary, crate::whisper::DEFAULT_GLOSSARY);
        assert_eq!(settings.whisper_style_example, crate::whisper::DEFAULT_STYLE_EXAMPLE);
    }

    #[test]
    fn api_mode_sends_the_glossary_inside_the_prompt_by_default() {
        let settings = AppSettings {
            whisper_style_example: "Hello there.".into(),
            whisper_glossary: "kubectl, pnpm".into(),
            ..AppSettings::default()
        };
        assert_eq!(
            settings.whisper_api_prompt_parts(),
            (Some("Hello there. Glossary: kubectl, pnpm.".into()), None)
        );
    }

    #[test]
    fn api_mode_with_hotwords_splits_style_and_glossary() {
        let settings = AppSettings {
            whisper_style_example: "Hello there.".into(),
            whisper_glossary: "kubectl,\npnpm".into(),
            whisper_api_hotwords: true,
            ..AppSettings::default()
        };
        assert_eq!(
            settings.whisper_api_prompt_parts(),
            (Some("Hello there.".into()), Some("kubectl, pnpm".into()))
        );
    }

    #[test]
    fn empty_prompt_boxes_send_no_prompt() {
        let settings = AppSettings {
            whisper_style_example: String::new(),
            whisper_glossary: String::new(),
            ..AppSettings::default()
        };
        assert_eq!(settings.whisper_initial_prompt(), None);
    }
}
