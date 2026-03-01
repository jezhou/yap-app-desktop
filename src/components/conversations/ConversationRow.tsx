import { useState } from "react";
import { useNavigate } from "react-router-dom";
import type { Conversation } from "../../types";

interface ConversationRowProps {
  conversation: Conversation;
  sessionId: string;
  onRename: (conversationId: string, title: string) => void;
  onDelete: (conversationId: string) => void;
}

function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

const STATUS_STYLES: Record<string, string> = {
  completed: "bg-success/10 text-success",
  analyzing: "bg-warning/10 text-warning",
  uploading: "bg-accent/10 text-accent",
  error: "bg-error/10 text-error",
};

export default function ConversationRow({
  conversation,
  sessionId,
  onRename,
  onDelete,
}: ConversationRowProps) {
  const navigate = useNavigate();
  const [editing, setEditing] = useState(false);
  const [editTitle, setEditTitle] = useState(conversation.title);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  function handleRenameSubmit() {
    const trimmed = editTitle.trim();
    if (trimmed && trimmed !== conversation.title) {
      onRename(conversation.id, trimmed);
    }
    setEditing(false);
  }

  const date = new Date(conversation.created_at).toLocaleDateString();
  const statusStyle = STATUS_STYLES[conversation.status] ?? "";

  return (
    <tr
      className="border-b border-border hover:bg-surface-hover cursor-pointer transition-colors"
      onClick={() => navigate(`/${sessionId}/${conversation.id}`)}
    >
      <td className="px-4 py-3 text-sm text-text-dim">
        {conversation.sequence_number}
      </td>
      <td className="px-4 py-3">
        {editing ? (
          <input
            type="text"
            value={editTitle}
            onChange={(e) => setEditTitle(e.target.value)}
            onBlur={handleRenameSubmit}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleRenameSubmit();
              if (e.key === "Escape") {
                setEditTitle(conversation.title);
                setEditing(false);
              }
            }}
            className="w-full bg-bg border border-border rounded px-2 py-1 text-text text-sm focus:outline-none focus:border-accent"
            autoFocus
            onClick={(e) => e.stopPropagation()}
          />
        ) : (
          <span className="text-sm text-text">{conversation.title}</span>
        )}
      </td>
      <td className="px-4 py-3">
        <span
          className={`text-xs px-2 py-0.5 rounded-full capitalize ${statusStyle}`}
        >
          {conversation.status}
        </span>
      </td>
      <td className="px-4 py-3 text-sm text-text-muted">{date}</td>
      <td className="px-4 py-3 text-sm text-text-muted font-mono">
        {formatDuration(conversation.duration_seconds)}
      </td>
      <td className="px-4 py-3">
        <div className="flex items-center gap-1" onClick={(e) => e.stopPropagation()}>
          <button
            onClick={() => setEditing(true)}
            className="p-1 text-text-dim hover:text-text text-xs rounded hover:bg-surface transition-colors"
          >
            Rename
          </button>
          {showDeleteConfirm ? (
            <>
              <button
                onClick={() => {
                  onDelete(conversation.id);
                  setShowDeleteConfirm(false);
                }}
                className="p-1 text-xs text-error hover:bg-error/10 rounded transition-colors"
              >
                Confirm
              </button>
              <button
                onClick={() => setShowDeleteConfirm(false)}
                className="p-1 text-xs text-text-dim hover:text-text rounded transition-colors"
              >
                Cancel
              </button>
            </>
          ) : (
            <button
              onClick={() => setShowDeleteConfirm(true)}
              className="p-1 text-text-dim hover:text-error text-xs rounded hover:bg-surface transition-colors"
            >
              Delete
            </button>
          )}
        </div>
      </td>
    </tr>
  );
}
