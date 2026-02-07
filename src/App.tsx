import { useState, useEffect } from "react";
import StatusIndicator from "./components/StatusIndicator";
import TranscriptionView from "./components/TranscriptionView";
import HistoryList from "./components/HistoryList";
import SettingsPage from "./components/SettingsPage";
import { useAppState } from "./hooks/useAppState";
import { isWhisperModelLoaded } from "./lib/commands";

type Page = "main" | "settings";

export default function App() {
  const { status, rawText, cleanedText, error } = useAppState();
  const [page, setPage] = useState<Page>("main");
  const [modelLoaded, setModelLoaded] = useState(true);

  useEffect(() => {
    isWhisperModelLoaded().then(setModelLoaded).catch(() => setModelLoaded(false));
  }, [page]);

  if (page === "settings") {
    return (
      <div className="min-h-screen bg-bg p-6">
        <div className="max-w-2xl mx-auto">
          <SettingsPage onBack={() => setPage("main")} />
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-bg p-6">
      <div className="max-w-2xl mx-auto">
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-2xl font-bold text-text">Speech AI Tool</h1>
          <div className="flex items-center gap-4">
            <StatusIndicator status={status} />
            <button
              onClick={() => setPage("settings")}
              className="px-3 py-1 text-sm text-text-muted hover:text-text transition-colors"
            >
              Settings
            </button>
          </div>
        </div>

        <div className="space-y-6">
          {!modelLoaded && (
            <div
              className="bg-yellow-900/30 border border-yellow-600 text-yellow-200 text-sm rounded-lg px-4 py-3 flex items-center justify-between cursor-pointer"
              onClick={() => setPage("settings")}
            >
              <span>
                No whisper model loaded. Go to Settings to download and load a model before transcribing.
              </span>
              <span className="text-yellow-400 text-xs ml-2">Settings &rarr;</span>
            </div>
          )}

          <div className="bg-surface rounded-lg p-6">
            <p className="text-text-muted text-sm">
              Press{" "}
              <kbd className="px-2 py-0.5 bg-primary rounded text-xs">
                Ctrl+Shift+Space
              </kbd>{" "}
              and hold while speaking. Release to transcribe.
            </p>
          </div>

          {error && (
            <div className="bg-error/10 text-error text-sm rounded-lg px-4 py-3">
              {error}
            </div>
          )}

          <TranscriptionView rawText={rawText} cleanedText={cleanedText} />

          <div className="bg-surface rounded-lg p-6">
            <HistoryList />
          </div>
        </div>
      </div>
    </div>
  );
}
