import { useSettings } from "../hooks/useSettings";
import AudioDeviceSelect from "./AudioDeviceSelect";
import HotkeyInput from "./HotkeyInput";
import WhisperSettings from "./WhisperSettings";
import LlmSettings from "./LlmSettings";
import type { AppSettings } from "../lib/types";

interface SettingsPageProps {
  onBack: () => void;
}

export default function SettingsPage({ onBack }: SettingsPageProps) {
  const { settings, loading, error, saveSettings } = useSettings();

  if (loading || !settings) {
    return <p className="text-text-muted text-sm">Loading settings...</p>;
  }

  const update = (partial: Partial<AppSettings>) => {
    saveSettings({ ...settings, ...partial });
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-bold text-text">Settings</h2>
        <button
          onClick={onBack}
          className="px-3 py-1 text-sm text-text-muted hover:text-text transition-colors"
        >
          Back
        </button>
      </div>

      {error && (
        <div className="bg-error/10 text-error text-sm rounded px-3 py-2">{error}</div>
      )}

      {/* Audio */}
      <section className="bg-surface rounded-lg p-4">
        <h3 className="text-sm font-medium text-text mb-3">Audio</h3>
        <AudioDeviceSelect
          onDeviceChange={(idx) => update({ audio_device_index: idx })}
        />
      </section>

      {/* Hotkey */}
      <section className="bg-surface rounded-lg p-4">
        <HotkeyInput
          value={settings.hotkey}
          onChange={(hotkey) => update({ hotkey })}
        />
      </section>

      {/* Whisper */}
      <section className="bg-surface rounded-lg p-4">
        <WhisperSettings
          settings={settings}
          onChange={saveSettings}
        />
      </section>

      {/* LLM */}
      <section className="bg-surface rounded-lg p-4">
        <LlmSettings
          settings={settings}
          onChange={saveSettings}
        />
      </section>

      {/* Output */}
      <section className="bg-surface rounded-lg p-4">
        <h3 className="text-sm font-medium text-text mb-3">Output</h3>
        <div className="space-y-3">
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={settings.auto_paste}
              onChange={(e) => update({ auto_paste: e.target.checked })}
              className="accent-accent"
            />
            <span className="text-sm text-text">Auto-paste after transcription</span>
          </label>
          {settings.auto_paste && (
            <div>
              <label className="block text-xs text-text-muted mb-1">
                Paste shortcut (e.g. Ctrl+V, Ctrl+Shift+V, Cmd+V)
              </label>
              <input
                type="text"
                value={settings.paste_shortcut}
                onChange={(e) => update({ paste_shortcut: e.target.value })}
                className="w-48 bg-bg border border-primary rounded px-3 py-2 text-text text-sm focus:outline-none focus:ring-1 focus:ring-accent"
              />
            </div>
          )}
        </div>
      </section>

      {/* History */}
      <section className="bg-surface rounded-lg p-4">
        <h3 className="text-sm font-medium text-text mb-3">History</h3>
        <div>
          <label className="block text-xs text-text-muted mb-1">Max items</label>
          <input
            type="number"
            value={settings.history_max_items}
            onChange={(e) => update({ history_max_items: Number(e.target.value) })}
            min={1}
            max={10000}
            className="w-24 bg-bg border border-primary rounded px-3 py-2 text-text text-sm focus:outline-none focus:ring-1 focus:ring-accent"
          />
        </div>
      </section>
    </div>
  );
}
