import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import type { Model, DiarizationStatus } from "../../types";
import {
  listAvailableModels,
  downloadModel,
  getDiarizationStatus,
  onModelDownloadProgress,
} from "../../services/settings";

const MODEL_INFO: Record<string, { ram: string; description: string }> = {
  base: { ram: "~1 GB RAM", description: "Fastest, good for clear audio" },
  small: { ram: "~2 GB RAM", description: "Recommended default" },
  medium: { ram: "~5 GB RAM", description: "Highest accuracy" },
};

export default function SetupWizard() {
  const navigate = useNavigate();
  const [models, setModels] = useState<Model[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>("small");
  const [downloading, setDownloading] = useState(false);
  const [downloadPercent, setDownloadPercent] = useState(0);
  const [downloadComplete, setDownloadComplete] = useState(false);
  const [diarization, setDiarization] = useState<DiarizationStatus | null>(
    null,
  );
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadModels();
  }, []);

  async function loadModels() {
    try {
      const [modelList, diarizationStatus] = await Promise.all([
        listAvailableModels(),
        getDiarizationStatus(),
      ]);
      setModels(modelList);
      setDiarization(diarizationStatus);

      const alreadyDownloaded = modelList.find((m) => m.downloaded);
      if (alreadyDownloaded) {
        setSelectedModel(alreadyDownloaded.name);
        setDownloadComplete(true);
      }
    } catch {
      setError("Failed to load model information.");
    }
  }

  async function handleDownload() {
    setDownloading(true);
    setDownloadPercent(0);
    setError(null);

    const unlisten = await onModelDownloadProgress((event) => {
      if (event.modelName === selectedModel) {
        setDownloadPercent(event.percent);
      }
    });

    try {
      await downloadModel(selectedModel);
      setDownloadComplete(true);
      await loadModels();
    } catch {
      setError("Download failed. Please try again.");
    } finally {
      setDownloading(false);
      unlisten();
    }
  }

  function handleGetStarted() {
    navigate("/");
  }

  return (
    <div className="flex items-center justify-center min-h-screen bg-bg p-8">
      <div className="w-full max-w-lg space-y-8">
        <div className="text-center">
          <h1 className="text-3xl font-bold text-accent">Yap</h1>
          <p className="mt-2 text-text-muted">
            Local voice conversation transcription
          </p>
        </div>

        <div className="bg-surface rounded-lg p-6 border border-border space-y-6">
          <div>
            <h2 className="text-lg font-semibold text-text">
              Select a transcription model
            </h2>
            <p className="mt-1 text-sm text-text-muted">
              Models run entirely on your device. No data leaves your computer.
            </p>
          </div>

          <div className="space-y-3">
            {models.map((model) => {
              const info = MODEL_INFO[model.name] ?? {
                ram: "",
                description: "",
              };
              return (
                <label
                  key={model.name}
                  className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${
                    selectedModel === model.name
                      ? "border-accent bg-accent/5"
                      : "border-border hover:border-border-hover"
                  }`}
                >
                  <input
                    type="radio"
                    name="model"
                    value={model.name}
                    checked={selectedModel === model.name}
                    onChange={() => setSelectedModel(model.name)}
                    disabled={downloading}
                    className="mt-1 accent-accent"
                  />
                  <div className="flex-1">
                    <div className="flex items-center justify-between">
                      <span className="font-medium text-text capitalize">
                        {model.name}
                      </span>
                      <span className="text-xs text-text-dim">
                        {model.size}
                      </span>
                    </div>
                    <p className="text-sm text-text-muted mt-0.5">
                      {info.description}
                    </p>
                    <p className="text-xs text-text-dim mt-0.5">{info.ram}</p>
                    {model.downloaded && (
                      <span className="text-xs text-success mt-1 inline-block">
                        Downloaded
                      </span>
                    )}
                  </div>
                </label>
              );
            })}

            {models.length === 0 && !error && (
              <p className="text-sm text-text-muted text-center py-4">
                Loading models...
              </p>
            )}
          </div>

          {downloading && (
            <div className="space-y-2">
              <div className="flex justify-between text-sm text-text-muted">
                <span>Downloading {selectedModel}...</span>
                <span>{downloadPercent}%</span>
              </div>
              <div className="w-full bg-bg rounded-full h-2">
                <div
                  className="bg-accent h-2 rounded-full transition-all duration-300"
                  style={{ width: `${downloadPercent}%` }}
                />
              </div>
            </div>
          )}

          {diarization && (
            <div className="text-sm space-y-1">
              <p className="text-text-muted font-medium">
                Speaker diarization models
              </p>
              <div className="flex items-center gap-2">
                <span
                  className={`w-2 h-2 rounded-full ${diarization.segmentationReady ? "bg-success" : "bg-text-dim"}`}
                />
                <span className="text-text-dim">Segmentation</span>
              </div>
              <div className="flex items-center gap-2">
                <span
                  className={`w-2 h-2 rounded-full ${diarization.embeddingReady ? "bg-success" : "bg-text-dim"}`}
                />
                <span className="text-text-dim">Speaker embedding</span>
              </div>
            </div>
          )}

          {error && <p className="text-sm text-error">{error}</p>}

          <div className="flex gap-3">
            {!downloadComplete ? (
              <button
                onClick={handleDownload}
                disabled={downloading || !selectedModel}
                className="flex-1 px-4 py-2.5 bg-accent text-bg font-medium rounded-lg hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {downloading ? "Downloading..." : "Download Model"}
              </button>
            ) : (
              <button
                onClick={handleGetStarted}
                className="flex-1 px-4 py-2.5 bg-accent text-bg font-medium rounded-lg hover:bg-accent-hover transition-colors"
              >
                Get Started
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
