import { useParams } from "react-router-dom";

export default function SessionDetailPage() {
  const { sessionId } = useParams<{ sessionId: string }>();

  return (
    <div>
      <h1 className="text-2xl font-bold text-text">Session: {sessionId}</h1>
    </div>
  );
}
