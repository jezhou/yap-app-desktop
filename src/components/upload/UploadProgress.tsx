import type { ConversationStatus } from "../../types";

interface UploadProgressProps {
  status: ConversationStatus;
  percent: number;
}

const STATUS_LABELS: Record<ConversationStatus, string> = {
  uploading: "Uploading",
  analyzing: "Transcribing",
  completed: "Completed",
  error: "Error",
};

export default function UploadProgress({
  status,
  percent,
}: UploadProgressProps) {
  const isActive = status === "uploading" || status === "analyzing";
  const isError = status === "error";
  const isComplete = status === "completed";

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between text-sm">
        <span
          className={`font-medium ${
            isError
              ? "text-error"
              : isComplete
                ? "text-success"
                : "text-text"
          }`}
        >
          {STATUS_LABELS[status]}
        </span>
        {isActive && <span className="text-text-muted">{percent}%</span>}
      </div>

      <div className="w-full bg-bg rounded-full h-2">
        <div
          className={`h-2 rounded-full transition-all duration-300 ${
            isError
              ? "bg-error"
              : isComplete
                ? "bg-success"
                : "bg-accent"
          }`}
          style={{ width: `${isComplete ? 100 : percent}%` }}
        />
      </div>
    </div>
  );
}
