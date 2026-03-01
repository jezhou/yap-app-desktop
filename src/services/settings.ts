import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Settings,
  Model,
  DiarizationStatus,
  ModelDownloadProgressEvent,
} from "../types";

export async function getSettings(): Promise<Settings[]> {
  return invoke("get_settings");
}

export async function updateSettings(
  key: string,
  value: string,
): Promise<{ updated: true }> {
  return invoke("update_settings", { key, value });
}

export async function listAvailableModels(): Promise<Model[]> {
  return invoke("list_available_models");
}

export async function downloadModel(modelName: string): Promise<void> {
  return invoke("download_model", { modelName });
}

export async function deleteModel(
  modelName: string,
): Promise<{ deleted: true }> {
  return invoke("delete_model", { modelName });
}

export async function getDiarizationStatus(): Promise<DiarizationStatus> {
  return invoke("get_diarization_status");
}

export function onModelDownloadProgress(
  callback: (event: ModelDownloadProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<ModelDownloadProgressEvent>(
    "model-download-progress",
    (event) => {
      callback(event.payload);
    },
  );
}
