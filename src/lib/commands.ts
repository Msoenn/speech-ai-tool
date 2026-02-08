import { invoke } from "@tauri-apps/api/core";
import type { AudioDevice, TranscriptionRecord, AppSettings } from "./types";

export async function listAudioDevices(): Promise<AudioDevice[]> {
  return invoke("list_audio_devices");
}

export async function startRecording(deviceIndex: number | null): Promise<void> {
  return invoke("start_recording", { deviceIndex });
}

export async function stopRecording(): Promise<number[]> {
  return invoke("stop_recording");
}

export async function transcribeAudio(wavBytes: number[]): Promise<string> {
  return invoke("transcribe_audio", { wavBytes });
}

export async function loadWhisperModel(modelName: string): Promise<void> {
  return invoke("load_whisper_model", { modelName });
}

export async function listWhisperModels(): Promise<{ name: string; size: string; downloaded: boolean }[]> {
  return invoke("list_whisper_models");
}

export async function isWhisperModelLoaded(): Promise<boolean> {
  return invoke("is_whisper_model_loaded");
}

export async function downloadWhisperModel(modelName: string): Promise<void> {
  return invoke("download_whisper_model", { modelName });
}

export async function cleanupText(rawText: string): Promise<string> {
  return invoke("cleanup_text", { rawText });
}

export async function testLlmConnection(): Promise<string> {
  return invoke("test_llm_connection");
}

export async function copyToClipboard(text: string): Promise<void> {
  return invoke("copy_to_clipboard", { text });
}

export async function pasteText(text: string): Promise<void> {
  return invoke("paste_text", { text });
}

export async function setHotkey(hotkey: string): Promise<void> {
  return invoke("set_hotkey", { hotkey });
}

export async function pauseHotkey(paused: boolean): Promise<void> {
  return invoke("pause_hotkey", { paused });
}

export async function getCurrentHotkey(): Promise<string> {
  return invoke("get_current_hotkey");
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function resetSettings(): Promise<AppSettings> {
  return invoke("reset_settings");
}

export async function getHistory(): Promise<TranscriptionRecord[]> {
  return invoke("get_history");
}

export async function deleteHistoryItem(id: string): Promise<void> {
  return invoke("delete_history_item", { id });
}

export async function clearHistory(): Promise<void> {
  return invoke("clear_history");
}

export async function testWhisperApi(): Promise<string> {
  return invoke("test_whisper_api");
}
