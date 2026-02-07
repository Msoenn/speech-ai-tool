import { useState, useEffect } from "react";
import { listAudioDevices, startRecording, stopRecording } from "../lib/commands";
import type { AudioDevice } from "../lib/types";

interface AudioDeviceSelectProps {
  onDeviceChange?: (index: number | null) => void;
}

export default function AudioDeviceSelect({ onDeviceChange }: AudioDeviceSelectProps) {
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const [recording, setRecording] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    listAudioDevices()
      .then((devs) => setDevices(devs))
      .catch((e) => setError(String(e)));
  }, []);

  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const val = e.target.value === "" ? null : Number(e.target.value);
    setSelectedIndex(val);
    onDeviceChange?.(val);
  };

  const toggleRecording = async () => {
    setError(null);
    setTestResult(null);
    try {
      if (recording) {
        const wavBytes = await stopRecording();
        setRecording(false);
        const sizeKb = Math.round(wavBytes.length / 1024);
        if (sizeKb < 1) {
          setTestResult("No audio captured. Check your microphone.");
        } else {
          setTestResult(`Mic works! Captured ${sizeKb} KB of audio.`);
        }
      } else {
        await startRecording(selectedIndex);
        setRecording(true);
      }
    } catch (e) {
      setError(String(e));
      setRecording(false);
    }
  };

  return (
    <div className="space-y-3">
      <label className="block text-sm font-medium text-text-muted">Microphone</label>
      <select
        value={selectedIndex ?? ""}
        onChange={handleChange}
        className="w-full bg-bg border border-primary rounded px-3 py-2 text-text text-sm focus:outline-none focus:ring-1 focus:ring-accent appearance-none cursor-pointer"
      >
        <option value="">System default</option>
        {devices.map((d) => (
          <option key={d.index} value={d.index}>
            {d.name}
          </option>
        ))}
      </select>

      <div className="flex items-center gap-3">
        <button
          onClick={toggleRecording}
          className={`px-4 py-2 rounded text-sm font-medium transition-colors ${
            recording
              ? "bg-recording text-white hover:bg-red-600 animate-pulse"
              : "bg-primary text-white hover:bg-blue-700"
          }`}
        >
          {recording ? "Stop" : "Test Microphone"}
        </button>
        {recording && (
          <span className="text-xs text-text-muted">Speak now, then click Stop...</span>
        )}
        {testResult && (
          <span className={`text-xs ${testResult.includes("works") ? "text-success" : "text-warning"}`}>
            {testResult}
          </span>
        )}
      </div>

      {error && <p className="text-error text-sm">{error}</p>}
    </div>
  );
}
