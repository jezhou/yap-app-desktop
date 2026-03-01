import { invoke } from "@tauri-apps/api/core";
import type {
  UploadAudioOutput,
  PlayAudioOutput,
  PauseAudioOutput,
  AudioPositionOutput,
} from "../types";

export async function uploadAudio(
  sessionId: string,
  filePath: string,
): Promise<UploadAudioOutput> {
  return invoke("upload_audio", { sessionId, filePath });
}

export async function playAudio(
  conversationId: string,
  seekSeconds?: number,
): Promise<PlayAudioOutput> {
  return invoke("play_audio", { conversationId, seekSeconds });
}

export async function pauseAudio(): Promise<PauseAudioOutput> {
  return invoke("pause_audio");
}

export async function getAudioPosition(): Promise<AudioPositionOutput> {
  return invoke("get_audio_position");
}
