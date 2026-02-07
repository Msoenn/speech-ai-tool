import { useState, useEffect, useCallback } from "react";
import { getSettings, saveSettings as saveSettingsCmd } from "../lib/commands";
import type { AppSettings } from "../lib/types";

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getSettings()
      .then((s) => {
        setSettings(s);
        setLoading(false);
      })
      .catch((e) => {
        setError(String(e));
        setLoading(false);
      });
  }, []);

  const saveSettings = useCallback(
    async (newSettings: AppSettings) => {
      try {
        await saveSettingsCmd(newSettings);
        setSettings(newSettings);
        setError(null);
      } catch (e) {
        setError(String(e));
      }
    },
    [],
  );

  return { settings, loading, error, saveSettings };
}
