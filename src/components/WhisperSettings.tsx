import { getDefaultWhisperPrompt } from "../lib/commands";
import type { AppSettings } from "../lib/types";
import ModelDownload from "./ModelDownload";

// Whisper keeps only the last 224 tokens of the prompt. Tech words cost
// about one token per 3 characters, so this is a rough estimate.
const PROMPT_TOKEN_LIMIT = 224;
const estimateTokens = (text: string) => Math.ceil(text.trim().length / 3);

interface WhisperSettingsProps {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
}

export default function WhisperSettings({ settings, onChange }: WhisperSettingsProps) {
  const styleExample = settings.whisper_style_example ?? "";
  const glossary = settings.whisper_glossary ?? "";
  const promptTokens = estimateTokens(styleExample) + estimateTokens(glossary);

  const resetStyleExample = async () => {
    const [defaultStyle] = await getDefaultWhisperPrompt();
    onChange({ ...settings, whisper_style_example: defaultStyle });
  };

  const resetGlossary = async () => {
    const [, defaultGlossary] = await getDefaultWhisperPrompt();
    onChange({ ...settings, whisper_glossary: defaultGlossary });
  };

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
          <div>
            <label className="block text-xs text-text-muted mb-1">Model</label>
            <input
              type="text"
              value={settings.whisper_api_model}
              onChange={(e) => onChange({ ...settings, whisper_api_model: e.target.value })}
              placeholder="whisper-1"
              className="w-full bg-bg border border-primary rounded px-3 py-2 text-text text-sm focus:outline-none focus:ring-1 focus:ring-accent"
            />
            <p className="text-xs text-text-muted mt-1">
              The model name sent to your Whisper API. For OpenAI use{" "}
              <code>whisper-1</code>; for a self-hosted server use the exact model id it has
              installed (e.g. <code>Systran/faster-whisper-large-v3</code>).
            </p>
          </div>
          <div>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={settings.whisper_api_hotwords ?? false}
                onChange={(e) =>
                  onChange({ ...settings, whisper_api_hotwords: e.target.checked })
                }
                className="accent-accent"
              />
              <span className="text-sm text-text">Send the glossary as hotwords</span>
            </label>
            <p className="text-xs text-text-muted mt-1">
              For Speaches and other faster-whisper servers. They apply the prompt to the
              first 30 seconds only, and hotwords to the whole recording. Leave it off for
              the OpenAI API.
            </p>
          </div>
        </div>
      )}

      <div className="space-y-3 border-t border-primary/20 pt-4">
        <div>
          <h4 className="text-sm font-medium text-text">Whisper Prompt</h4>
          <p className="text-xs text-text-muted mt-1">
            Whisper does not follow instructions. It copies the style of this text and
            the spelling of these words. Empty both boxes to send no prompt.
          </p>
        </div>

        <div>
          <div className="flex items-center justify-between mb-1">
            <label className="block text-xs text-text-muted">Glossary</label>
            <button
              onClick={resetGlossary}
              className="text-xs text-text-muted hover:text-text transition-colors"
            >
              Reset to default
            </button>
          </div>
          <textarea
            value={glossary}
            onChange={(e) => onChange({ ...settings, whisper_glossary: e.target.value })}
            rows={4}
            placeholder="Kubernetes, kubectl, PostgreSQL"
            className="w-full bg-bg border border-primary rounded px-3 py-2 text-text text-sm focus:outline-none focus:ring-1 focus:ring-accent resize-y"
          />
          <p className="text-xs text-text-muted mt-1">
            Names and words that Whisper must spell right. Separate them with commas.
          </p>
        </div>

        <div>
          <div className="flex items-center justify-between mb-1">
            <label className="block text-xs text-text-muted">Style Example</label>
            <button
              onClick={resetStyleExample}
              className="text-xs text-text-muted hover:text-text transition-colors"
            >
              Reset to default
            </button>
          </div>
          <textarea
            value={styleExample}
            onChange={(e) => onChange({ ...settings, whisper_style_example: e.target.value })}
            rows={2}
            className="w-full bg-bg border border-primary rounded px-3 py-2 text-text text-sm focus:outline-none focus:ring-1 focus:ring-accent resize-y"
          />
          <p className="text-xs text-text-muted mt-1">
            A short sample of how the text must look: punctuation, capitals, no filler
            words.
          </p>
        </div>

        <p
          className={`text-xs ${
            promptTokens > PROMPT_TOKEN_LIMIT ? "text-error" : "text-text-muted"
          }`}
        >
          About {promptTokens} of {PROMPT_TOKEN_LIMIT} tokens.
          {promptTokens > PROMPT_TOKEN_LIMIT &&
            " The prompt is too long. Whisper drops the start of the style example first."}
        </p>
      </div>
    </div>
  );
}
