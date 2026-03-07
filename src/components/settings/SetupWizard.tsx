import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import {
  getSettings,
  updateSettings,
  validateApiKey,
} from "../../services/settings";

export default function SetupWizard() {
  const navigate = useNavigate();
  const [apiKey, setApiKey] = useState("");
  const [validating, setValidating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasKey, setHasKey] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    checkExistingKey();
  }, []);

  async function checkExistingKey() {
    try {
      const settings = await getSettings();
      const keySetting = settings.find((s) => s.key === "deepgram_api_key");
      if (keySetting && keySetting.value) {
        setHasKey(true);
      }
    } catch {
      // ignore
    } finally {
      setLoading(false);
    }
  }

  async function handleSave() {
    const trimmed = apiKey.trim();
    if (!trimmed) {
      setError("Please enter an API key.");
      return;
    }

    setValidating(true);
    setError(null);

    try {
      const result = await validateApiKey(trimmed);
      if (result.valid) {
        await updateSettings("deepgram_api_key", trimmed);
        setHasKey(true);
      } else {
        setError(result.error || "Invalid API key. Please check and try again.");
      }
    } catch {
      setError("Failed to validate API key. Check your internet connection.");
    } finally {
      setValidating(false);
    }
  }

  function handleGetStarted() {
    navigate("/");
  }

  if (loading) {
    return null;
  }

  return (
    <div className="flex items-center justify-center min-h-screen bg-bg p-8">
      <div className="w-full max-w-lg space-y-8">
        <div className="text-center">
          <h1 className="text-3xl font-bold text-accent">Yap</h1>
          <p className="mt-2 text-text-muted">
            Voice conversation transcription
          </p>
        </div>

        <div className="bg-surface rounded-lg p-6 border border-border space-y-6">
          <div>
            <h2 className="text-lg font-semibold text-text">
              Connect to Deepgram
            </h2>
            <p className="mt-1 text-sm text-text-muted">
              Yap uses Deepgram for high-quality speech-to-text with speaker
              identification. Enter your API key to get started.
            </p>
          </div>

          {!hasKey ? (
            <>
              <div className="space-y-2">
                <label
                  htmlFor="setup-api-key"
                  className="block text-sm font-medium text-text"
                >
                  Deepgram API Key
                </label>
                <input
                  id="setup-api-key"
                  type="password"
                  value={apiKey}
                  onChange={(e) => {
                    setApiKey(e.target.value);
                    setError(null);
                  }}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") handleSave();
                  }}
                  placeholder="Enter your API key..."
                  className="w-full bg-bg border border-border rounded-lg px-3 py-2 text-sm text-text placeholder-text-dim focus:outline-none focus:border-accent font-mono"
                  autoFocus
                />
                <p className="text-xs text-text-dim">
                  Get a free API key at{" "}
                  <a
                    href="https://console.deepgram.com/signup"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-accent hover:underline"
                  >
                    console.deepgram.com
                  </a>
                </p>
              </div>

              {error && <p className="text-sm text-error">{error}</p>}

              <button
                onClick={handleSave}
                disabled={validating || !apiKey.trim()}
                className="w-full px-4 py-2.5 bg-accent text-bg font-medium rounded-lg hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {validating ? "Validating..." : "Save & Continue"}
              </button>
            </>
          ) : (
            <>
              <div className="flex items-center gap-2 text-success">
                <span className="text-sm font-medium">
                  API key configured
                </span>
              </div>

              <button
                onClick={handleGetStarted}
                className="w-full px-4 py-2.5 bg-accent text-bg font-medium rounded-lg hover:bg-accent-hover transition-colors"
              >
                Get Started
              </button>
            </>
          )}
        </div>

        <p className="text-xs text-text-dim text-center">
          Audio is sent to Deepgram for transcription. Your API key is stored
          locally and never shared.
        </p>
      </div>
    </div>
  );
}
