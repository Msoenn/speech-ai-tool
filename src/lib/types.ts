export type PipelineStatus =
  | "idle"
  | "recording"
  | "transcribing"
  | "cleaning"
  | "done"
  | "error";

export interface AudioDevice {
  index: number;
  name: string;
}

export interface WhisperModel {
  name: string;
  size: string;
  downloaded: boolean;
  path?: string;
}

export interface TranscriptionRecord {
  id: string;
  raw_text: string;
  cleaned_text: string;
  created_at: string;
  duration_secs: number;
  model_used: string;
}

export interface FewShotExample {
  input: string;
  output: string;
}

export interface LlmConfig {
  endpoint: string;
  model: string;
  system_prompt: string;
  api_type: "ollama" | "openai";
  few_shot_examples: FewShotExample[];
}

export interface AppSettings {
  audio_device_index: number | null;
  hotkey: string;
  whisper_mode: "local" | "api";
  whisper_model: string;
  whisper_api_endpoint: string;
  whisper_api_key: string;
  llm: LlmConfig;
  auto_paste: boolean;
  paste_shortcut: string;
  history_max_items: number;
}

export interface PipelineStatusEvent {
  status: PipelineStatus;
  raw_text?: string;
  cleaned_text?: string;
  error?: string;
}
