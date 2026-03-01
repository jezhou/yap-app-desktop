import { invoke } from "@tauri-apps/api/core";
import type { Conversation, ConversationDetail } from "../types";

export async function listConversations(
  sessionId: string,
): Promise<Conversation[]> {
  return invoke("list_conversations", { sessionId });
}

export async function getConversationDetail(
  conversationId: string,
): Promise<ConversationDetail> {
  return invoke("get_conversation_detail", { conversationId });
}

export async function renameConversation(
  conversationId: string,
  title: string,
): Promise<{ updated: true }> {
  return invoke("rename_conversation", { conversationId, title });
}

export async function deleteConversation(
  conversationId: string,
): Promise<{ deleted: true }> {
  return invoke("delete_conversation", { conversationId });
}

export async function updateSpeakerRole(
  conversationId: string,
  speakerLabel: string,
  displayName: string,
): Promise<{ updated: true }> {
  return invoke("update_speaker_role", {
    conversationId,
    speakerLabel,
    displayName,
  });
}
