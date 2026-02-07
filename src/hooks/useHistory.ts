import { useState, useEffect, useCallback } from "react";
import {
  getHistory,
  deleteHistoryItem,
  clearHistory as clearHistoryCmd,
} from "../lib/commands";
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
