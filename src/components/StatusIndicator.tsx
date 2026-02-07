import type { PipelineStatus } from "../lib/types";

const statusConfig: Record<PipelineStatus, { color: string; label: string }> = {
  idle: { color: "bg-text-muted", label: "Idle" },
  recording: { color: "bg-recording animate-pulse", label: "Recording..." },
  transcribing: { color: "bg-processing animate-pulse", label: "Transcribing..." },
  cleaning: { color: "bg-processing animate-pulse", label: "Cleaning up..." },
  done: { color: "bg-success", label: "Done" },
  error: { color: "bg-error", label: "Error" },
};

interface StatusIndicatorProps {
  status: PipelineStatus;
}

export default function StatusIndicator({ status }: StatusIndicatorProps) {
  const config = statusConfig[status];

  return (
    <div className="flex items-center gap-2">
      <div className={`w-3 h-3 rounded-full ${config.color}`} />
      <span className="text-sm text-text-muted">{config.label}</span>
    </div>
  );
}
