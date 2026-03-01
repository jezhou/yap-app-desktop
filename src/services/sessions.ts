import { invoke } from "@tauri-apps/api/core";
import type { Session } from "../types";

export async function listSessions(
  limit?: number,
  offset?: number,
): Promise<Session[]> {
  return invoke("list_sessions", { limit, offset });
}

export async function createSession(
  title: string,
  description?: string,
): Promise<{ sessionId: string }> {
  return invoke("create_session", { title, description });
}

export async function updateSession(
  sessionId: string,
  title?: string,
  description?: string,
): Promise<{ updated: true }> {
  return invoke("update_session", { sessionId, title, description });
}

export async function deleteSession(
  sessionId: string,
): Promise<{ deleted: true }> {
  return invoke("delete_session", { sessionId });
}
