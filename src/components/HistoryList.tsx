import { useState } from "react";
import { useHistory } from "../hooks/useHistory";
import { copyToClipboard } from "../lib/commands";

export default function HistoryList() {
  const { records, loading, deleteItem, clearHistory } = useHistory();
  const [expandedId, setExpandedId] = useState<string | null>(null);

  if (loading) {
    return <p className="text-text-muted text-sm">Loading history...</p>;
  }

  if (records.length === 0) {
    return <p className="text-text-muted text-sm">No transcription history yet.</p>;
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-text-muted">
          History ({records.length} items)
        </h3>
        <button
          onClick={clearHistory}
          className="px-3 py-1 text-xs text-error hover:bg-error/10 rounded transition-colors"
        >
          Clear All
        </button>
      </div>

      <div className="space-y-2 max-h-96 overflow-y-auto">
        {records.map((record) => {
          const isExpanded = expandedId === record.id;
          const preview = record.cleaned_text.slice(0, 100);

          return (
            <div key={record.id} className="bg-bg rounded p-3">
              <div
                className="flex items-start justify-between cursor-pointer"
                onClick={() => setExpandedId(isExpanded ? null : record.id)}
              >
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-text truncate">
                    {preview}
                    {record.cleaned_text.length > 100 && "..."}
                  </p>
                  <div className="flex gap-3 mt-1">
                    <span className="text-xs text-text-muted">
                      {new Date(record.created_at).toLocaleString()}
                    </span>
                    <span className="text-xs text-text-muted">
                      {record.duration_secs.toFixed(1)}s
                    </span>
                    <span className="text-xs text-text-muted">{record.model_used}</span>
                  </div>
                </div>
                <span className="text-text-muted text-xs ml-2">{isExpanded ? "▲" : "▼"}</span>
              </div>

              {isExpanded && (
                <div className="mt-3 space-y-3 border-t border-primary/20 pt-3">
                  <div>
                    <h4 className="text-xs font-medium text-text-muted mb-1">Raw</h4>
                    <p className="text-sm text-text whitespace-pre-wrap">{record.raw_text}</p>
                  </div>
                  <div>
                    <h4 className="text-xs font-medium text-text-muted mb-1">Cleaned</h4>
                    <p className="text-sm text-text whitespace-pre-wrap">{record.cleaned_text}</p>
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        copyToClipboard(record.cleaned_text);
                      }}
                      className="px-3 py-1 text-xs bg-primary rounded hover:bg-blue-700 transition-colors"
                    >
                      Copy Cleaned
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        deleteItem(record.id);
                      }}
                      className="px-3 py-1 text-xs text-error hover:bg-error/10 rounded transition-colors"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
