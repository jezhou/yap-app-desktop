import type { Conversation } from "../../types";
import ConversationRow from "./ConversationRow";

interface ConversationListProps {
  conversations: Conversation[];
  sessionId: string;
  onRename: (conversationId: string, title: string) => void;
  onDelete: (conversationId: string) => void;
}

export default function ConversationList({
  conversations,
  sessionId,
  onRename,
  onDelete,
}: ConversationListProps) {
  if (conversations.length === 0) {
    return null;
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full">
        <thead>
          <tr className="border-b border-border text-left">
            <th className="px-4 py-2 text-xs font-medium text-text-dim uppercase tracking-wider w-12">
              #
            </th>
            <th className="px-4 py-2 text-xs font-medium text-text-dim uppercase tracking-wider">
              Title
            </th>
            <th className="px-4 py-2 text-xs font-medium text-text-dim uppercase tracking-wider w-28">
              Status
            </th>
            <th className="px-4 py-2 text-xs font-medium text-text-dim uppercase tracking-wider w-28">
              Date
            </th>
            <th className="px-4 py-2 text-xs font-medium text-text-dim uppercase tracking-wider w-20">
              Duration
            </th>
            <th className="px-4 py-2 text-xs font-medium text-text-dim uppercase tracking-wider w-32">
              Actions
            </th>
          </tr>
        </thead>
        <tbody>
          {conversations.map((conversation) => (
            <ConversationRow
              key={conversation.id}
              conversation={conversation}
              sessionId={sessionId}
              onRename={onRename}
              onDelete={onDelete}
            />
          ))}
        </tbody>
      </table>
    </div>
  );
}
