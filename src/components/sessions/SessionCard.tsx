import { useState } from "react";
import { useNavigate } from "react-router-dom";
import type { Session } from "../../types";

interface SessionCardProps {
  session: Session;
  onRename: (sessionId: string, title: string) => void;
  onDelete: (sessionId: string) => void;
}

export default function SessionCard({
  session,
  onRename,
  onDelete,
}: SessionCardProps) {
  const navigate = useNavigate();
  const [editing, setEditing] = useState(false);
  const [editTitle, setEditTitle] = useState(session.title);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  function handleRenameSubmit() {
    const trimmed = editTitle.trim();
    if (trimmed && trimmed !== session.title) {
      onRename(session.id, trimmed);
    }
    setEditing(false);
  }

  function handleDelete() {
    onDelete(session.id);
    setShowDeleteConfirm(false);
  }

  const date = new Date(session.created_at).toLocaleDateString();

  return (
    <div className="p-4 bg-surface rounded-lg border border-border hover:border-border-hover transition-colors">
      <div className="flex items-start justify-between gap-2">
        <div
          className="flex-1 min-w-0 cursor-pointer"
          onClick={() => navigate(`/${session.id}`)}
        >
          {editing ? (
            <input
              type="text"
              value={editTitle}
              onChange={(e) => setEditTitle(e.target.value)}
              onBlur={handleRenameSubmit}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleRenameSubmit();
                if (e.key === "Escape") {
                  setEditTitle(session.title);
                  setEditing(false);
                }
              }}
              className="w-full bg-bg border border-border rounded px-2 py-1 text-text text-sm focus:outline-none focus:border-accent"
              autoFocus
              onClick={(e) => e.stopPropagation()}
            />
          ) : (
            <h3 className="font-medium text-text truncate">{session.title}</h3>
          )}
          {session.description && (
            <p className="text-sm text-text-muted mt-1 truncate">
              {session.description}
            </p>
          )}
          <div className="flex items-center gap-3 mt-2 text-xs text-text-dim">
            <span>{date}</span>
            <span>
              {session.conversation_count ?? 0} conversation
              {(session.conversation_count ?? 0) !== 1 ? "s" : ""}
            </span>
          </div>
        </div>

        <div className="flex items-center gap-1 flex-shrink-0">
          <button
            onClick={(e) => {
              e.stopPropagation();
              setEditing(true);
            }}
            className="p-1.5 text-text-dim hover:text-text rounded hover:bg-surface-hover transition-colors text-xs"
            title="Rename"
          >
            Rename
          </button>
          {showDeleteConfirm ? (
            <div className="flex items-center gap-1" onClick={(e) => e.stopPropagation()}>
              <button
                onClick={handleDelete}
                className="px-2 py-1 text-xs text-error hover:bg-error/10 rounded transition-colors"
              >
                Confirm
              </button>
              <button
                onClick={() => setShowDeleteConfirm(false)}
                className="px-2 py-1 text-xs text-text-dim hover:text-text rounded transition-colors"
              >
                Cancel
              </button>
            </div>
          ) : (
            <button
              onClick={(e) => {
                e.stopPropagation();
                setShowDeleteConfirm(true);
              }}
              className="p-1.5 text-text-dim hover:text-error rounded hover:bg-surface-hover transition-colors text-xs"
              title="Delete"
            >
              Delete
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
