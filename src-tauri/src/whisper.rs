use serde::Serialize;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

use crate::error::AppError;

const WHISPER_MODELS: &[(&str, &str, &str)] = &[
    ("tiny", "75 MB", "ggml-tiny.bin"),
    ("base", "142 MB", "ggml-base.bin"),
    ("small", "466 MB", "ggml-small.bin"),
    ("medium", "1.5 GB", "ggml-medium.bin"),
    ("large-v3-turbo", "1.6 GB", "ggml-large-v3-turbo.bin"),
];

fn huggingface_url(filename: &str) -> String {
    format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
        filename
    )
}

pub fn get_models_dir() -> Result<PathBuf, AppError> {
    let dir = dirs::data_dir()
        .ok_or_else(|| AppError::Whisper("Cannot determine data directory".into()))?
        .join("speech-ai-tool")
        .join("models");
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Whisper(e.to_string()))?;
    Ok(dir)
}

#[derive(Debug, Clone, Serialize)]
pub struct WhisperModelInfo {
    pub name: String,
    pub size: String,
    pub downloaded: bool,
    pub path: Option<String>,
}

pub fn list_models() -> Result<Vec<WhisperModelInfo>, AppError> {
    let models_dir = get_models_dir()?;
    let mut result = Vec::new();

    for &(name, size, filename) in WHISPER_MODELS {
        let path = models_dir.join(filename);
        let downloaded = path.exists();
        result.push(WhisperModelInfo {
            name: name.to_string(),
            size: size.to_string(),
            downloaded,
            path: if downloaded {
                Some(path.to_string_lossy().into_owned())
            } else {
                None
            },
        });
    }

    Ok(result)
}

pub async fn download_model(app: AppHandle, model_name: &str) -> Result<(), AppError> {
    let (_, _, filename) = WHISPER_MODELS
        .iter()
        .find(|(name, _, _)| *name == model_name)
        .ok_or_else(|| AppError::Whisper(format!("Unknown model: {}", model_name)))?;

    let models_dir = get_models_dir()?;
    let target_path = models_dir.join(filename);

    if target_path.exists() {
        return Ok(());
    }

    let url = huggingface_url(filename);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Whisper(format!("Download failed: {}", e)))?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let temp_path = target_path.with_extension("part");
    let mut file =
        std::fs::File::create(&temp_path).map_err(|e| AppError::Whisper(e.to_string()))?;

    use futures::StreamExt;
    use std::io::Write;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AppError::Whisper(format!("Download error: {}", e)))?;
        file.write_all(&chunk)
            .map_err(|e| AppError::Whisper(e.to_string()))?;
        downloaded += chunk.len() as u64;

        let progress = if total_size > 0 {
            (downloaded as f64 / total_size as f64 * 100.0) as u32
        } else {
            0
        };

        let _ = app.emit(
            "model-download-progress",
            serde_json::json!({
                "model": model_name,
                "progress": progress,
                "downloaded": downloaded,
                "total": total_size,
            }),
        );
    }

    std::fs::rename(&temp_path, &target_path)
        .map_err(|e| AppError::Whisper(e.to_string()))?;

    Ok(())
}

pub struct WhisperEngine {
    ctx: Mutex<Option<whisper_rs::WhisperContext>>,
}

impl WhisperEngine {
    pub fn new() -> Self {
        Self {
            ctx: Mutex::new(None),
        }
    }

    pub fn is_model_loaded(&self) -> bool {
        self.ctx.lock().unwrap().is_some()
    }

    pub fn load_model(&self, model_name: &str) -> Result<(), AppError> {
        let (_, _, filename) = WHISPER_MODELS
            .iter()
            .find(|(name, _, _)| *name == model_name)
            .ok_or_else(|| AppError::Whisper(format!("Unknown model: {}", model_name)))?;

        let models_dir = get_models_dir()?;
        let model_path = models_dir.join(filename);

        if !model_path.exists() {
            return Err(AppError::Whisper(format!(
                "Model not downloaded: {}",
                model_name
            )));
        }

        let ctx = whisper_rs::WhisperContext::new_with_params(
            model_path.to_str().unwrap(),
            whisper_rs::WhisperContextParameters::default(),
        )
        .map_err(|e| AppError::Whisper(format!("Failed to load model: {}", e)))?;

        *self.ctx.lock().unwrap() = Some(ctx);
        Ok(())
    }

    pub fn transcribe(&self, wav_bytes: &[u8]) -> Result<String, AppError> {
        let guard = self.ctx.lock().unwrap();
        let ctx = guard
            .as_ref()
            .ok_or_else(|| AppError::Whisper("No model loaded".into()))?;

        let samples = decode_wav_to_samples(wav_bytes)?;

        let mut state = ctx
            .create_state()
            .map_err(|e| AppError::Whisper(format!("Failed to create state: {}", e)))?;

        let mut params = whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(num_cpus::get() as i32);
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state
            .full(params, &samples)
            .map_err(|e| AppError::Whisper(format!("Transcription failed: {}", e)))?;

        let num_segments = state.full_n_segments()
            .map_err(|e| AppError::Whisper(format!("Failed to get segments: {}", e)))?;
        let mut text = String::new();

        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                text.push_str(&segment);
            }
        }

        Ok(text.trim().to_string())
    }
}

pub async fn transcribe_via_api(
    endpoint: &str,
    api_key: &str,
    wav_bytes: &[u8],
) -> Result<String, AppError> {
    let url = format!(
        "{}/v1/audio/transcriptions",
        endpoint.trim_end_matches('/')
    );

    let part = reqwest::multipart::Part::bytes(wav_bytes.to_vec())
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| AppError::Whisper(e.to_string()))?;

    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("model", "whisper-1");

    let client = reqwest::Client::new();
    let mut req = client.post(&url).multipart(form);

    if !api_key.is_empty() {
        req = req.bearer_auth(api_key);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| AppError::Whisper(format!("API request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::Whisper(format!("API error {}: {}", status, text)));
    }

    #[derive(serde::Deserialize)]
    struct TranscriptionResponse {
        text: String,
    }

    let parsed: TranscriptionResponse = resp
        .json()
        .await
        .map_err(|e| AppError::Whisper(format!("Parse error: {}", e)))?;

    Ok(parsed.text.trim().to_string())
}

fn decode_wav_to_samples(wav_bytes: &[u8]) -> Result<Vec<f32>, AppError> {
    let cursor = Cursor::new(wav_bytes);
    let mut reader =
        hound::WavReader::new(cursor).map_err(|e| AppError::Whisper(format!("Invalid WAV: {}", e)))?;

    let spec = reader.spec();
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => reader
            .samples::<i16>()
            .filter_map(|s| s.ok())
            .map(|s| s as f32 / i16::MAX as f32)
            .collect(),
        hound::SampleFormat::Float => reader.samples::<f32>().filter_map(|s| s.ok()).collect(),
    };

    Ok(samples)
}
