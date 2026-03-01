import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { TranscriptionProgressEvent } from "../types";

export async function startTranscription(
  conversationId: string,
): Promise<{ status: "analyzing" }> {
  return invoke("start_transcription", { conversationId });
}

export async function cancelTranscription(
  conversationId: string,
): Promise<{ status: "error"; reason: "cancelled" }> {
  return invoke("cancel_transcription", { conversationId });
}

export function onTranscriptionProgress(
  callback: (event: TranscriptionProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<TranscriptionProgressEvent>(
    "transcription-progress",
    (event) => {
      callback(event.payload);
    },
  );
}
