import { useState, useEffect, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { uploadAudio } from "../../services/audio";
import type { UploadAudioOutput } from "../../types";

const SUPPORTED_FORMATS = [".mp3", ".wav", ".m4a", ".ogg", ".webm"];

interface UploadDropzoneProps {
  sessionId: string;
  onUploadStart?: (result: UploadAudioOutput) => void;
  onError?: (error: string) => void;
}

export default function UploadDropzone({
  sessionId,
  onUploadStart,
  onError,
}: UploadDropzoneProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [uploading, setUploading] = useState(false);

  const handleFile = useCallback(
    async (filePath: string) => {
      const ext = filePath.slice(filePath.lastIndexOf(".")).toLowerCase();
      if (!SUPPORTED_FORMATS.includes(ext)) {
        onError?.(
          `Unsupported format "${ext}". Supported: ${SUPPORTED_FORMATS.join(", ")}`,
        );
        return;
      }

      setUploading(true);
      try {
        const result = await uploadAudio(sessionId, filePath);
        onUploadStart?.(result);
      } catch (err) {
        onError?.(err instanceof Error ? err.message : "Upload failed");
      } finally {
        setUploading(false);
      }
    },
    [sessionId, onUploadStart, onError],
  );

  // Listen for Tauri native drag-drop events (provides full file paths)
  useEffect(() => {
    const appWindow = getCurrentWebviewWindow();
    const unlisten = appWindow.onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        setIsDragging(true);
      } else if (event.payload.type === "leave") {
        setIsDragging(false);
      } else if (event.payload.type === "drop") {
        setIsDragging(false);
        const paths = event.payload.paths;
        if (paths.length > 0) {
          handleFile(paths[0]);
        }
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [handleFile]);

  async function handleBrowse() {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Audio Files",
          extensions: ["mp3", "wav", "m4a", "ogg", "webm"],
        },
      ],
    });

    if (selected) {
      await handleFile(selected);
    }
  }

  return (
    <div
      className={`flex flex-col items-center justify-center p-8 rounded-lg border-2 border-dashed transition-colors cursor-pointer ${
        isDragging
          ? "border-accent bg-accent/5"
          : "border-border hover:border-border-hover"
      } ${uploading ? "opacity-50 pointer-events-none" : ""}`}
      onClick={handleBrowse}
    >
      <div className="text-center space-y-3">
        <div className="text-4xl text-text-dim">
          {uploading ? "\u23F3" : "\u2B06"}
        </div>
        <div>
          <p className="text-text font-medium">
            {uploading
              ? "Uploading..."
              : "Drop audio file here or click to browse"}
          </p>
          <p className="text-sm text-text-muted mt-1">
            Supported formats: {SUPPORTED_FORMATS.join(", ")}
          </p>
        </div>
      </div>
    </div>
  );
}
