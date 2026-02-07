import { useState, useCallback } from "react";
import { useTauriEvent } from "./useTauriEvent";
import type { PipelineStatus, PipelineStatusEvent } from "../lib/types";

export function useAppState() {
  const [status, setStatus] = useState<PipelineStatus>("idle");
  const [rawText, setRawText] = useState<string>("");
  const [cleanedText, setCleanedText] = useState<string>("");
  const [error, setError] = useState<string | null>(null);

  const handler = useCallback((event: PipelineStatusEvent) => {
    setStatus(event.status);
    if (event.raw_text) setRawText(event.raw_text);
    if (event.cleaned_text) setCleanedText(event.cleaned_text);
    if (event.error) setError(event.error);
    if (event.status === "idle" || event.status === "recording") {
      setError(null);
    }
  }, []);

  useTauriEvent<PipelineStatusEvent>("pipeline-status", handler);

  return { status, rawText, cleanedText, error, setStatus };
}
