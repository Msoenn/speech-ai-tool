import { useState, useEffect, useCallback } from "react";
import {
  getHistory,
  deleteHistoryItem,
  clearHistory as clearHistoryCmd,
} from "../lib/commands";
import { useTauriEvent } from "./useTauriEvent";
import type { TranscriptionRecord } from "../lib/types";

export function useHistory() {
  const [records, setRecords] = useState<TranscriptionRecord[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const items = await getHistory();
      setRecords(items);
    } catch (e) {
      console.error("Failed to load history:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  // Reload when the backend saves a new transcription, so entries recorded
  // while the dashboard is open appear immediately (newest-first) instead of
  // only after a restart.
  useTauriEvent<unknown>("history-updated", refresh);

  const deleteItem = useCallback(
    async (id: string) => {
      await deleteHistoryItem(id);
      await refresh();
    },
    [refresh],
  );

  const clearHistory = useCallback(async () => {
    await clearHistoryCmd();
    setRecords([]);
  }, []);

  return { records, loading, deleteItem, clearHistory, refresh };
}
