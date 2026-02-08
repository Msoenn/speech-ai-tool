import { useState, useEffect, useCallback } from "react";
import { listWhisperModels, downloadWhisperModel, loadWhisperModel } from "../lib/commands";
import { useTauriEvent } from "../hooks/useTauriEvent";

interface ModelInfo {
  name: string;
  size: string;
  description: string;
  downloaded: boolean;
}

interface DownloadProgress {
  model: string;
  progress: number;
  downloaded: number;
  total: number;
}

interface ModelDownloadProps {
  currentModel: string;
  onModelLoaded?: (model: string) => void;
}

export default function ModelDownload({ currentModel, onModelLoaded }: ModelDownloadProps) {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [downloading, setDownloading] = useState<string | null>(null);
  const [progress, setProgress] = useState(0);
  const [loading, setLoading] = useState<string | null>(null);
  const [loadedModel, setLoadedModel] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    listWhisperModels()
      .then((m) => {
        setModels(m);
        // Only mark as loaded if the current model is actually downloaded
        const current = m.find((x) => x.name === currentModel);
        if (current?.downloaded) {
          setLoadedModel(currentModel);
        }
      })
      .catch((e) => setError(String(e)));
  }, [currentModel]);

  const progressHandler = useCallback((event: DownloadProgress) => {
    setProgress(event.progress);
  }, []);

  useTauriEvent<DownloadProgress>("model-download-progress", progressHandler);

  const handleDownload = async (name: string) => {
    setError(null);
    setDownloading(name);
    setProgress(0);
    try {
      await downloadWhisperModel(name);
      setModels((prev) =>
        prev.map((m) => (m.name === name ? { ...m, downloaded: true } : m)),
      );
    } catch (e) {
      setError(String(e));
    } finally {
      setDownloading(null);
    }
  };

  const handleLoad = async (name: string) => {
    setError(null);
    setLoading(name);
    try {
      await loadWhisperModel(name);
      setLoadedModel(name);
      onModelLoaded?.(name);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(null);
    }
  };

  return (
    <div className="space-y-3">
      <h3 className="text-sm font-medium text-text-muted">Whisper Models</h3>
      {!loadedModel && (
        <p className="text-xs text-warning">No model loaded. Download and load a model to enable transcription.</p>
      )}

      <div className="space-y-2">
        {models.map((model) => {
          const isLoaded = model.name === loadedModel;

          return (
            <div
              key={model.name}
              className={`flex items-center justify-between bg-bg rounded px-3 py-2 ${
                isLoaded ? "ring-1 ring-accent" : ""
              }`}
            >
              <div>
                <div>
                  <span className="text-sm text-text font-medium">{model.name}</span>
                  <span className="text-xs text-text-muted ml-2">{model.size}</span>
                  {isLoaded && (
                    <span className="text-xs text-accent ml-2">(loaded)</span>
                  )}
                  {model.downloaded && !isLoaded && (
                    <span className="text-xs text-success ml-2">(downloaded)</span>
                  )}
                </div>
                <div className="text-xs text-text-muted">{model.description}</div>
              </div>

              <div className="flex gap-2">
                {!model.downloaded && downloading !== model.name && (
                  <button
                    onClick={() => handleDownload(model.name)}
                    className="px-3 py-1 text-xs bg-primary rounded hover:bg-blue-700 transition-colors"
                  >
                    Download
                  </button>
                )}

                {downloading === model.name && (
                  <div className="flex items-center gap-2">
                    <div className="w-24 h-2 bg-surface rounded-full overflow-hidden">
                      <div
                        className="h-full bg-accent transition-all"
                        style={{ width: `${progress}%` }}
                      />
                    </div>
                    <span className="text-xs text-text-muted">{progress}%</span>
                  </div>
                )}

                {model.downloaded && !isLoaded && downloading !== model.name && (
                  <button
                    onClick={() => handleLoad(model.name)}
                    disabled={loading === model.name}
                    className="px-3 py-1 text-xs bg-success/20 text-success rounded hover:bg-success/30 disabled:opacity-50 transition-colors"
                  >
                    {loading === model.name ? "Loading..." : "Load"}
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {error && <p className="text-error text-sm">{error}</p>}
    </div>
  );
}
