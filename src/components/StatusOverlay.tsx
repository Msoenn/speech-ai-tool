import { useState, useCallback, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTauriEvent } from "../hooks/useTauriEvent";
import type { PipelineStatus, PipelineStatusEvent } from "../lib/types";

export default function StatusOverlay() {
  const [status, setStatus] = useState<PipelineStatus>("recording");
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    document.documentElement.classList.add("overlay-window");
  }, []);

  const handler = useCallback((event: PipelineStatusEvent) => {
    setStatus(event.status);
    setVisible(true);
  }, []);

  useTauriEvent<PipelineStatusEvent>("pipeline-status", handler);

  useEffect(() => {
    if (status === "done") {
      const timer = setTimeout(() => setVisible(false), 2000);
      return () => clearTimeout(timer);
    }
    if (status === "error") {
      const timer = setTimeout(() => setVisible(false), 3000);
      return () => clearTimeout(timer);
    }
  }, [status]);

  const handleMouseDown = useCallback(async () => {
    try {
      await getCurrentWindow().startDragging();
    } catch {}
  }, []);

  if (!visible) return null;

  return (
    <div
      onMouseDown={handleMouseDown}
      className="h-screen w-screen flex items-center justify-center select-none cursor-grab active:cursor-grabbing"
      style={{ background: "#1a1a2e" }}
    >
      {status === "recording" && (
        <svg className="w-5 h-5" viewBox="0 0 24 24" fill="#ef4444">
          <rect x="3" y="3" width="18" height="18" rx="3">
            <animate attributeName="opacity" values="1;0.4;1" dur="1.2s" repeatCount="indefinite" />
          </rect>
        </svg>
      )}
      {(status === "transcribing" || status === "cleaning") && (
        <svg className="w-5 h-5 animate-spin" viewBox="0 0 24 24" fill="none">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="#eab308" strokeWidth="3" />
          <path fill="#eab308" opacity="0.8" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
        </svg>
      )}
      {status === "done" && (
        <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="#4ade80" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
          <polyline points="20 6 9 17 4 12" />
        </svg>
      )}
      {status === "error" && (
        <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="#f87171" strokeWidth="3" strokeLinecap="round">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      )}
    </div>
  );
}
