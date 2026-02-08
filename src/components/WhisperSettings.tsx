import type { AppSettings } from "../lib/types";
import ModelDownload from "./ModelDownload";

interface WhisperSettingsProps {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
}

export default function WhisperSettings({ settings, onChange }: WhisperSettingsProps) {
  return (
    <div className="space-y-4">
      <h3 className="text-sm font-medium text-text">Whisper Configuration</h3>

      <div className="flex gap-4">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="radio"
            name="whisper_mode"
            checked={settings.whisper_mode === "local"}
            onChange={() => onChange({ ...settings, whisper_mode: "local" })}
            className="accent-accent"
          />
          <span className="text-sm text-text">Local</span>
        </label>
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="radio"
            name="whisper_mode"
            checked={settings.whisper_mode === "api"}
            onChange={() => onChange({ ...settings, whisper_mode: "api" })}
            className="accent-accent"
          />
          <span className="text-sm text-text">API</span>
        </label>
      </div>

      <div>
        <label className="block text-xs text-text-muted mb-1">Language</label>
        <select
          value={settings.whisper_language}
          onChange={(e) => onChange({ ...settings, whisper_language: e.target.value })}
          className="w-full bg-bg border border-primary rounded px-3 py-2 text-text text-sm focus:outline-none focus:ring-1 focus:ring-accent"
        >
          <option value="auto">Auto-detect</option>
          <option value="en">English</option>
          <option value="zh">Chinese</option>
          <option value="de">German</option>
          <option value="es">Spanish</option>
          <option value="fr">French</option>
          <option value="it">Italian</option>
          <option value="ja">Japanese</option>
          <option value="ko">Korean</option>
          <option value="pt">Portuguese</option>
          <option value="ru">Russian</option>
          <option value="ar">Arabic</option>
          <option value="hi">Hindi</option>
          <option value="nl">Dutch</option>
          <option value="pl">Polish</option>
          <option value="sv">Swedish</option>
          <option value="tr">Turkish</option>
          <option value="uk">Ukrainian</option>
          <option value="vi">Vietnamese</option>
          <option value="th">Thai</option>
          <option value="id">Indonesian</option>
        </select>
      </div>

      {settings.whisper_mode === "local" ? (
        <ModelDownload
          currentModel={settings.whisper_model}
          onModelLoaded={(model) => onChange({ ...settings, whisper_model: model })}
        />
      ) : (
        <div className="space-y-3">
          <div>
            <label className="block text-xs text-text-muted mb-1">API Endpoint</label>
            <input
              type="text"
              value={settings.whisper_api_endpoint}
              onChange={(e) => onChange({ ...settings, whisper_api_endpoint: e.target.value })}
              placeholder="https://api.openai.com"
              className="w-full bg-bg border border-primary rounded px-3 py-2 text-text text-sm focus:outline-none focus:ring-1 focus:ring-accent"
            />
          </div>
          <div>
            <label className="block text-xs text-text-muted mb-1">API Key</label>
            <input
              type="password"
              value={settings.whisper_api_key}
              onChange={(e) => onChange({ ...settings, whisper_api_key: e.target.value })}
              placeholder="sk-..."
              className="w-full bg-bg border border-primary rounded px-3 py-2 text-text text-sm focus:outline-none focus:ring-1 focus:ring-accent"
            />
          </div>
        </div>
      )}
    </div>
  );
}
