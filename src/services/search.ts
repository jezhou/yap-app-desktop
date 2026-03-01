import { invoke } from "@tauri-apps/api/core";
import type { SearchResult } from "../types";

export async function searchConversations(
  query: string,
  limit?: number,
): Promise<SearchResult[]> {
  return invoke("search_conversations", { query, limit });
}
