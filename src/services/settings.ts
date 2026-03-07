import { invoke } from "@tauri-apps/api/core";
import type { Settings, ValidateApiKeyResult } from "../types";

export async function getSettings(): Promise<Settings[]> {
  return invoke("get_settings");
}

export async function updateSettings(
  key: string,
  value: string,
): Promise<{ updated: true }> {
  return invoke("update_settings", { key, value });
}

export async function validateApiKey(
  apiKey: string,
): Promise<ValidateApiKeyResult> {
  return invoke("validate_api_key", { apiKey });
}
