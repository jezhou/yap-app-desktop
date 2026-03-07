import { useState, useEffect } from "react";
import {
  getSettings,
  updateSettings,
  validateApiKey,
} from "../services/settings";

export default function SettingsPage() {
  const [apiKey, setApiKey] = useState("");
  const [hasSavedKey, setHasSavedKey] = useState(false);
  const [showKey, setShowKey] = useState(false);
  const [validating, setValidating] = useState(false);
  const [status, setStatus] = useState<"idle" | "valid" | "invalid" | "saved">(
    "idle",
  );
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadApiKey();
  }, []);

  async function loadApiKey() {
    try {
      const settings = await getSettings();
      const keySetting = settings.find((s) => s.key === "deepgram_api_key");
      if (keySetting && keySetting.value) {
        setApiKey(keySetting.value);
        setHasSavedKey(true);
        setStatus("valid");
      }
    } catch {
      setError("Failed to load settings.");
    }
  }

  async function handleSave() {
    const trimmed = apiKey.trim();
    if (!trimmed) {
      setError("API key cannot be empty.");
      return;
    }

    setValidating(true);
    setError(null);
    setStatus("idle");

    try {
      const result = await validateApiKey(trimmed);
      if (result.valid) {
        await updateSettings("deepgram_api_key", trimmed);
        setHasSavedKey(true);
        setStatus("saved");
        setShowKey(false);
      } else {
        setStatus("invalid");
        setError(result.error || "Invalid API key.");
      }
    } catch {
      setError("Failed to validate API key. Check your internet connection.");
    } finally {
      setValidating(false);
    }
  }

  async function handleRemove() {
    try {
      await updateSettings("deepgram_api_key", "");
      setApiKey("");
      setHasSavedKey(false);
      setStatus("idle");
      setShowKey(false);
    } catch {
      setError("Failed to remove API key.");
    }
  }

  return (
    <div className="space-y-8 max-w-2xl">
      <h1 className="text-2xl font-bold text-text">Settings</h1>

      {error && (
        <p className="text-sm text-error bg-error/10 rounded-lg px-4 py-2">
          {error}
        </p>
      )}

      {/* Deepgram API Key */}
      <section className="space-y-4">
        <div>
          <h2 className="text-lg font-semibold text-text">Deepgram API Key</h2>
          <p className="text-sm text-text-muted mt-1">
            Yap uses{" "}
            <a
              href="https://deepgram.com"
              target="_blank"
              rel="noopener noreferrer"
              className="text-accent hover:underline"
            >
              Deepgram
            </a>{" "}
            for speech-to-text transcription with speaker diarization. Get a
            free API key at{" "}
            <a
              href="https://console.deepgram.com/signup"
              target="_blank"
              rel="noopener noreferrer"
              className="text-accent hover:underline"
            >
              console.deepgram.com
            </a>
            .
          </p>
        </div>

        <div className="bg-surface rounded-lg p-4 border border-border space-y-4">
          <div className="space-y-2">
            <label
              htmlFor="api-key"
              className="block text-sm font-medium text-text"
            >
              API Key
            </label>
            <div className="flex gap-2">
              <div className="relative flex-1">
                <input
                  id="api-key"
                  type={showKey ? "text" : "password"}
                  value={apiKey}
                  onChange={(e) => {
                    setApiKey(e.target.value);
                    setStatus("idle");
                    setError(null);
                  }}
                  placeholder="Enter your Deepgram API key..."
                  className="w-full bg-bg border border-border rounded-lg px-3 py-2 pr-16 text-sm text-text placeholder-text-dim focus:outline-none focus:border-accent font-mono"
                />
                <button
                  type="button"
                  onClick={() => setShowKey(!showKey)}
                  className="absolute right-2 top-1/2 -translate-y-1/2 text-xs text-text-muted hover:text-text px-2 py-1"
                >
                  {showKey ? "Hide" : "Show"}
                </button>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-3">
            <button
              onClick={handleSave}
              disabled={validating || !apiKey.trim()}
              className="px-4 py-2 text-sm bg-accent text-bg font-medium rounded-lg hover:bg-accent-hover disabled:opacity-50 transition-colors"
            >
              {validating ? "Validating..." : "Validate & Save"}
            </button>

            {hasSavedKey && (
              <button
                onClick={handleRemove}
                className="px-4 py-2 text-sm text-error hover:bg-error/10 rounded-lg transition-colors"
              >
                Remove
              </button>
            )}

            {status === "valid" && !validating && hasSavedKey && (
              <span className="text-xs text-success">Configured</span>
            )}
            {status === "saved" && (
              <span className="text-xs text-success">Saved successfully</span>
            )}
            {status === "invalid" && (
              <span className="text-xs text-error">Invalid key</span>
            )}
          </div>
        </div>

        <p className="text-xs text-text-dim">
          Audio is sent to Deepgram for transcription. Your API key is stored
          locally and never shared with anyone else.
        </p>
      </section>
    </div>
  );
}
