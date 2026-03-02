import { useState, useEffect, useCallback } from "react";
import SessionList from "../components/sessions/SessionList";
import {
  listSessions,
  createSession,
  updateSession,
  deleteSession,
} from "../services/sessions";
import { onModelDownloadProgress } from "../services/settings";
import type { Session } from "../types";

export default function SessionsPage() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [creating, setCreating] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [downloadPercent, setDownloadPercent] = useState<number | null>(null);

  const loadSessions = useCallback(async () => {
    try {
      const result = await listSessions();
      setSessions(result);
    } catch {
      setError("Failed to load sessions.");
    }
  }, []);

  useEffect(() => {
    loadSessions();
  }, [loadSessions]);

  // Listen for background model download progress
  useEffect(() => {
    const unlisten = onModelDownloadProgress((event) => {
      if (event.percent >= 100) {
        setDownloadPercent(null);
      } else {
        setDownloadPercent(event.percent);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function handleCreate() {
    const trimmed = newTitle.trim();
    if (!trimmed) return;

    try {
      await createSession(trimmed);
      setNewTitle("");
      setCreating(false);
      await loadSessions();
    } catch {
      setError("Failed to create session.");
    }
  }

  async function handleRename(sessionId: string, title: string) {
    try {
      await updateSession(sessionId, title);
      await loadSessions();
    } catch {
      setError("Failed to rename session.");
    }
  }

  async function handleDelete(sessionId: string) {
    try {
      await deleteSession(sessionId);
      await loadSessions();
    } catch {
      setError("Failed to delete session.");
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-text">Sessions</h1>
        {!creating && (
          <button
            onClick={() => setCreating(true)}
            className="px-4 py-2 text-sm bg-accent text-bg font-medium rounded-lg hover:bg-accent-hover transition-colors"
          >
            New Session
          </button>
        )}
      </div>

      {downloadPercent !== null && (
        <div className="bg-accent/10 border border-accent/30 rounded-lg px-4 py-3">
          <p className="text-sm text-accent font-medium">
            Downloading transcription model... {downloadPercent}%
          </p>
          <div className="mt-2 h-1.5 bg-surface-hover rounded-full overflow-hidden">
            <div
              className="h-full bg-accent rounded-full transition-all duration-300"
              style={{ width: `${downloadPercent}%` }}
            />
          </div>
        </div>
      )}

      {error && (
        <p className="text-sm text-error bg-error/10 rounded-lg px-4 py-2">
          {error}
        </p>
      )}

      {creating && (
        <div className="flex gap-2">
          <input
            type="text"
            value={newTitle}
            onChange={(e) => setNewTitle(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleCreate();
              if (e.key === "Escape") {
                setCreating(false);
                setNewTitle("");
              }
            }}
            placeholder="Session title..."
            className="flex-1 bg-bg border border-border rounded-lg px-3 py-2 text-sm text-text placeholder-text-dim focus:outline-none focus:border-accent"
            autoFocus
          />
          <button
            onClick={handleCreate}
            className="px-4 py-2 text-sm bg-accent text-bg font-medium rounded-lg hover:bg-accent-hover transition-colors"
          >
            Create
          </button>
          <button
            onClick={() => {
              setCreating(false);
              setNewTitle("");
            }}
            className="px-4 py-2 text-sm text-text-muted hover:text-text rounded-lg hover:bg-surface-hover transition-colors"
          >
            Cancel
          </button>
        </div>
      )}

      {sessions.length === 0 && !creating ? (
        <div className="flex flex-col items-center justify-center py-16 border-2 border-dashed border-border rounded-lg">
          <p className="text-text-muted">No sessions yet</p>
          <p className="text-sm text-text-dim mt-1">
            Create your first session to start transcribing
          </p>
          <button
            onClick={() => setCreating(true)}
            className="mt-4 px-4 py-2 text-sm bg-accent text-bg font-medium rounded-lg hover:bg-accent-hover transition-colors"
          >
            Create your first session
          </button>
        </div>
      ) : (
        <SessionList
          sessions={sessions}
          onRename={handleRename}
          onDelete={handleDelete}
        />
      )}
    </div>
  );
}
