import { useState, useEffect } from "react";
import type { Model, DiarizationStatus } from "../types";
import {
  listAvailableModels,
  downloadModel,
  deleteModel,
  getDiarizationStatus,
  onModelDownloadProgress,
} from "../services/settings";

export default function SettingsPage() {
  const [models, setModels] = useState<Model[]>([]);
  const [diarization, setDiarization] = useState<DiarizationStatus | null>(
    null,
  );
  const [downloadingModel, setDownloadingModel] = useState<string | null>(null);
  const [downloadPercent, setDownloadPercent] = useState(0);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  async function loadData() {
    try {
      const [modelList, diarizationStatus] = await Promise.all([
        listAvailableModels(),
        getDiarizationStatus(),
      ]);
      setModels(modelList);
      setDiarization(diarizationStatus);
    } catch {
      setError("Failed to load settings.");
    }
  }

  async function handleDownload(modelName: string) {
    setDownloadingModel(modelName);
    setDownloadPercent(0);
    setError(null);

    const unlisten = await onModelDownloadProgress((event) => {
      if (event.modelName === modelName) {
        setDownloadPercent(event.percent);
      }
    });

    try {
      await downloadModel(modelName);
      await loadData();
    } catch {
      setError(`Failed to download ${modelName}.`);
    } finally {
      setDownloadingModel(null);
      unlisten();
    }
  }

  async function handleDelete(modelName: string) {
    setError(null);
    try {
      await deleteModel(modelName);
      await loadData();
    } catch {
      setError(`Failed to delete ${modelName}.`);
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

      {/* Transcription Models */}
      <section className="space-y-4">
        <div>
          <h2 className="text-lg font-semibold text-text">
            Transcription Models
          </h2>
          <p className="text-sm text-text-muted mt-1">
            Whisper ONNX models for local speech-to-text. All processing stays
            on your device.
          </p>
        </div>

        <div className="space-y-3">
          {models.map((model) => (
            <div
              key={model.name}
              className="flex items-center justify-between p-4 bg-surface rounded-lg border border-border"
            >
              <div>
                <div className="flex items-center gap-2">
                  <span className="font-medium text-text capitalize">
                    {model.name}
                  </span>
                  <span className="text-xs text-text-dim px-2 py-0.5 bg-bg rounded">
                    {model.quality_tier}
                  </span>
                </div>
                <p className="text-sm text-text-muted mt-0.5">{model.size}</p>
              </div>

              <div className="flex items-center gap-2">
                {model.downloaded ? (
                  <>
                    <span className="text-xs text-success">Downloaded</span>
                    <button
                      onClick={() => handleDelete(model.name)}
                      className="px-3 py-1.5 text-sm text-error hover:bg-error/10 rounded transition-colors"
                    >
                      Delete
                    </button>
                  </>
                ) : downloadingModel === model.name ? (
                  <div className="flex items-center gap-2">
                    <div className="w-24 bg-bg rounded-full h-1.5">
                      <div
                        className="bg-accent h-1.5 rounded-full transition-all duration-300"
                        style={{ width: `${downloadPercent}%` }}
                      />
                    </div>
                    <span className="text-xs text-text-muted">
                      {downloadPercent}%
                    </span>
                  </div>
                ) : (
                  <button
                    onClick={() => handleDownload(model.name)}
                    disabled={downloadingModel !== null}
                    className="px-3 py-1.5 text-sm bg-accent text-bg rounded hover:bg-accent-hover disabled:opacity-50 transition-colors"
                  >
                    Download
                  </button>
                )}
              </div>
            </div>
          ))}

          {models.length === 0 && (
            <p className="text-sm text-text-muted text-center py-4">
              Loading models...
            </p>
          )}
        </div>
      </section>

      {/* Diarization Status */}
      {diarization && (
        <section className="space-y-4">
          <div>
            <h2 className="text-lg font-semibold text-text">
              Speaker Diarization
            </h2>
            <p className="text-sm text-text-muted mt-1">
              Models for identifying different speakers in conversations.
            </p>
          </div>

          <div className="bg-surface rounded-lg p-4 border border-border space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-sm text-text">
                Segmentation (pyannote)
              </span>
              <span
                className={`text-xs ${diarization.segmentationReady ? "text-success" : "text-text-dim"}`}
              >
                {diarization.segmentationReady ? "Ready" : "Not available"}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-text">
                Speaker Embedding (3dspeaker)
              </span>
              <span
                className={`text-xs ${diarization.embeddingReady ? "text-success" : "text-text-dim"}`}
              >
                {diarization.embeddingReady ? "Ready" : "Not available"}
              </span>
            </div>
          </div>
        </section>
      )}
    </div>
  );
}
