use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::AppError;
use crate::history::TranscriptionRecord;
use crate::settings::WhisperMode;
use crate::tray;
use crate::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PipelineStatus {
    Recording,
    Transcribing,
    Cleaning,
    Done,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineStatusEvent {
    pub status: PipelineStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleaned_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn emit_status(app: &AppHandle, event: &PipelineStatusEvent) {
    let _ = app.emit("pipeline-status", event);
}

pub async fn run_pipeline(app: AppHandle) -> Result<(), AppError> {
    let start_time = std::time::Instant::now();

    // 1. Stop recording and get WAV bytes
    tray::set_tray_status(&app, "processing");
    emit_status(
        &app,
        &PipelineStatusEvent {
            status: PipelineStatus::Transcribing,
            raw_text: None,
            cleaned_text: None,
            error: None,
        },
    );

    let state = app.state::<AppState>();

    let wav_bytes = state.recorder.lock().unwrap().stop_recording()?;
    let duration_secs = start_time.elapsed().as_secs_f64();

    // 2. Transcribe
    let settings = state.settings.lock().unwrap().clone();

    let raw_text = match settings.whisper_mode {
        WhisperMode::Local => state.whisper.transcribe(&wav_bytes, &settings.whisper_language)?,
        WhisperMode::Api => {
            crate::whisper::transcribe_via_api(
                &settings.whisper_api_endpoint,
                &settings.whisper_api_key,
                &wav_bytes,
                &settings.whisper_language,
            )
            .await?
        }
    };

    if raw_text.trim().is_empty() {
        tray::set_tray_status(&app, "idle");
        tray::hide_overlay(&app);
        emit_status(
            &app,
            &PipelineStatusEvent {
                status: PipelineStatus::Error,
                raw_text: None,
                cleaned_text: None,
                error: Some("No speech detected".into()),
            },
        );
        return Ok(());
    }

    emit_status(
        &app,
        &PipelineStatusEvent {
            status: PipelineStatus::Cleaning,
            raw_text: Some(raw_text.clone()),
            cleaned_text: None,
            error: None,
        },
    );

    // 3. LLM cleanup (graceful degradation: skip if unavailable)
    let cleaned_text = match crate::llm::cleanup_text(&settings.llm, &raw_text).await {
        Ok(cleaned) => cleaned,
        Err(e) => {
            eprintln!("LLM cleanup failed, using raw text: {}", e);
            raw_text.clone()
        }
    };

    // 4. Output
    crate::output::copy_and_paste(&app, &cleaned_text, settings.auto_paste, &settings.paste_shortcut)?;

    tray::set_tray_status(&app, "done");
    emit_status(
        &app,
        &PipelineStatusEvent {
            status: PipelineStatus::Done,
            raw_text: Some(raw_text.clone()),
            cleaned_text: Some(cleaned_text.clone()),
            error: None,
        },
    );

    // Reset tray and hide overlay after 2 seconds
    let app_for_reset = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        tray::set_tray_status(&app_for_reset, "idle");
        tray::hide_overlay(&app_for_reset);
    });

    // 5. Save to history
    let record = TranscriptionRecord {
        id: uuid::Uuid::new_v4().to_string(),
        raw_text,
        cleaned_text,
        created_at: chrono::Utc::now().to_rfc3339(),
        duration_secs,
        model_used: settings.whisper_model.clone(),
    };

    if let Err(e) = state.history.insert(&record) {
        eprintln!("Failed to save history: {}", e);
    }
    let _ = state.history.prune(settings.history_max_items);

    Ok(())
}
