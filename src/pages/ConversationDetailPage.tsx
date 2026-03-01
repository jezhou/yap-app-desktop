import { useParams } from "react-router-dom";

export default function ConversationDetailPage() {
  const { sessionId, conversationId } = useParams<{
    sessionId: string;
    conversationId: string;
  }>();

  return (
    <div>
      <h1 className="text-2xl font-bold text-text">
        Conversation: {conversationId}
      </h1>
      <p className="text-text-muted">Session: {sessionId}</p>
    </div>
  );
}
