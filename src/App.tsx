import { useState, useEffect, useCallback } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import StatusIndicator from "./components/StatusIndicator";
import TranscriptionView from "./components/TranscriptionView";
import HistoryList from "./components/HistoryList";
import SettingsPage from "./components/SettingsPage";
import StatusOverlay from "./components/StatusOverlay";
import { useAppState } from "./hooks/useAppState";
import { useSettings } from "./hooks/useSettings";
import { useTauriEvent } from "./hooks/useTauriEvent";
import { applyPalette, isPaletteId } from "./lib/palettes";
import {
  isWhisperModelLoaded,
  checkAccessibilityPermission,
  requestAccessibilityPermission,
  openAccessibilitySettings,
  restartHotkeyListener,
} from "./lib/commands";

const isOverlay =
  new URLSearchParams(window.location.search).get("window") === "overlay";

export default function App() {
  if (isOverlay) {
    return <StatusOverlay />;
  }

  return <Dashboard />;
}

type Page = "main" | "settings";

function Dashboard() {
  const { status, rawText, cleanedText, error } = useAppState();
  const { settings } = useSettings();
  const [page, setPage] = useState<Page>("main");
  const [modelLoaded, setModelLoaded] = useState(true);
  const [update, setUpdate] = useState<Update | null>(null);
  const [updating, setUpdating] = useState(false);
  const [permissionOk, setPermissionOk] = useState(true);

  useEffect(() => {
    isWhisperModelLoaded().then(setModelLoaded).catch(() => setModelLoaded(false));
  }, [page]);

  // Settings is the authoritative palette source; re-apply whenever it changes
  // (also corrects the localStorage-cached guess from first paint).
  useEffect(() => {
    if (isPaletteId(settings?.palette)) applyPalette(settings.palette);
  }, [settings?.palette]);

  useEffect(() => {
    check().then((u) => setUpdate(u)).catch(console.error);
  }, []);

  useEffect(() => {
    checkAccessibilityPermission().then(setPermissionOk).catch(() => {});
  }, []);

  // While the banner is up, poll so it dismisses itself (and the hotkey
  // listener restarts) the moment the user flips the toggle in System
  // Settings — no relaunch needed.
  useEffect(() => {
    if (permissionOk) return;
    const id = setInterval(() => {
      restartHotkeyListener()
        .then((ok) => {
          if (ok) setPermissionOk(true);
        })
        .catch(() => {});
    }, 2000);
    return () => clearInterval(id);
  }, [permissionOk]);

  // The backend emits this when it detects the permission is missing at startup.
  const onPermissionRequired = useCallback(() => setPermissionOk(false), []);
  useTauriEvent<string>("permission-required", onPermissionRequired);

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
          {!permissionOk && (
            <div className="bg-error/10 border border-error text-error text-sm rounded-lg px-4 py-3 space-y-2">
              <p>
                Accessibility permission is required for the global hotkey and
                auto-paste to work.
              </p>
              <p>
                If you recently updated the app, System Settings may show Speech
                AI Tool already enabled — that entry is stale: remove it with the
                &minus; button (or toggle it off and on), then re-add the app.
                This banner disappears as soon as access is granted.
              </p>
              <div className="flex flex-wrap items-center gap-2">
                <button
                  onClick={async () => {
                    await requestAccessibilityPermission();
                    await openAccessibilitySettings();
                  }}
                  className="px-3 py-1 bg-error hover:bg-error-hover text-white text-xs rounded transition-colors"
                >
                  Grant Access
                </button>
                <button
                  onClick={() => openAccessibilitySettings()}
                  className="px-3 py-1 bg-surface hover:bg-primary text-text text-xs rounded transition-colors"
                >
                  Open Settings
                </button>
                <button
                  onClick={() =>
                    restartHotkeyListener().then(setPermissionOk).catch(() => {})
                  }
                  className="px-3 py-1 bg-surface hover:bg-primary text-text text-xs rounded transition-colors"
                >
                  Re-check
                </button>
              </div>
            </div>
          )}

          {update && (
            <div className="bg-primary/10 border border-primary text-accent text-sm rounded-lg px-4 py-3 flex items-center justify-between">
              <span>
                Update available: v{update.version}
              </span>
              <button
                disabled={updating}
                onClick={async () => {
                  setUpdating(true);
                  try {
                    await update.downloadAndInstall();
                    await relaunch();
                  } catch (e) {
                    console.error("Update failed:", e);
                    setUpdating(false);
                  }
                }}
                className="px-3 py-1 bg-primary hover:bg-primary-hover disabled:opacity-50 text-white text-xs rounded transition-colors"
              >
                {updating ? "Installing..." : "Install & Restart"}
              </button>
            </div>
          )}

          {settings?.whisper_mode === "local" && !modelLoaded && (
            <div
              className="bg-warning/10 border border-warning text-warning text-sm rounded-lg px-4 py-3 flex items-center justify-between cursor-pointer"
              onClick={() => setPage("settings")}
            >
              <span>
                No whisper model loaded. Go to Settings to download and load a model before transcribing.
              </span>
              <span className="text-warning text-xs ml-2">Settings &rarr;</span>
            </div>
          )}

          <div className="bg-surface rounded-lg p-6">
            <p className="text-text-muted text-sm">
              Press and hold your configured hotkey while speaking. Release to
              transcribe. You can change the hotkey in Settings.
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
