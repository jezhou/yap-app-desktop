import type { ConversationStatus } from "../../types";

interface StatusBadgeProps {
  status: ConversationStatus;
}

const STATUS_CONFIG: Record<
  ConversationStatus,
  { label: string; className: string }
> = {
  completed: { label: "Completed", className: "bg-success/10 text-success" },
  analyzing: { label: "Analyzing", className: "bg-warning/10 text-warning" },
  uploading: { label: "Uploading", className: "bg-accent/10 text-accent" },
  error: { label: "Error", className: "bg-error/10 text-error" },
};

export default function StatusBadge({ status }: StatusBadgeProps) {
  const config = STATUS_CONFIG[status];

  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${config.className}`}
    >
      {config.label}
    </span>
  );
}
