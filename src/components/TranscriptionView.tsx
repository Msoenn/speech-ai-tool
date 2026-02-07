import { useState } from "react";
import { copyToClipboard } from "../lib/commands";

interface TranscriptionViewProps {
  rawText: string;
  cleanedText: string;
}

function CopyButton({ text, label }: { text: string; label: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await copyToClipboard(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch (e) {
      console.error("Copy failed:", e);
    }
  };

  return (
    <button
      onClick={handleCopy}
      disabled={!text}
      className="px-3 py-1 text-xs bg-primary rounded hover:bg-blue-700 disabled:opacity-50 transition-colors"
    >
      {copied ? "Copied!" : label}
    </button>
  );
}

export default function TranscriptionView({ rawText, cleanedText }: TranscriptionViewProps) {
  if (!rawText && !cleanedText) return null;

  return (
    <div className="space-y-4">
      {rawText && (
        <div className="bg-surface rounded-lg p-4">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-sm font-medium text-text-muted">Raw Transcription</h3>
            <CopyButton text={rawText} label="Copy Raw" />
          </div>
          <p className="text-text text-sm whitespace-pre-wrap">{rawText}</p>
        </div>
      )}

      {cleanedText && (
        <div className="bg-surface rounded-lg p-4 border border-primary/30">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-sm font-medium text-text-muted">Cleaned Text</h3>
            <CopyButton text={cleanedText} label="Copy Cleaned" />
          </div>
          <p className="text-text text-sm whitespace-pre-wrap">{cleanedText}</p>
        </div>
      )}
    </div>
  );
}
