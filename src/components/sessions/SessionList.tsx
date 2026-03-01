import type { Session } from "../../types";
import SessionCard from "./SessionCard";

interface SessionListProps {
  sessions: Session[];
  onRename: (sessionId: string, title: string) => void;
  onDelete: (sessionId: string) => void;
}

export default function SessionList({
  sessions,
  onRename,
  onDelete,
}: SessionListProps) {
  if (sessions.length === 0) {
    return null;
  }

  return (
    <div className="space-y-3">
      {sessions.map((session) => (
        <SessionCard
          key={session.id}
          session={session}
          onRename={onRename}
          onDelete={onDelete}
        />
      ))}
    </div>
  );
}
