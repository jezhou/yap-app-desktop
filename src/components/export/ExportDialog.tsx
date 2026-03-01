import { useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { exportMarkdown, exportPdf } from "../../services/export";

type ExportFormat = "markdown" | "pdf";

interface ExportDialogProps {
  conversationId: string;
  conversationTitle: string;
  onClose: () => void;
}

export default function ExportDialog({
  conversationId,
  conversationTitle,
  onClose,
}: ExportDialogProps) {
  const [format, setFormat] = useState<ExportFormat>("markdown");
  const [exporting, setExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  async function handleExport() {
    setError(null);
    setExporting(true);

    try {
      const ext = format === "markdown" ? "md" : "pdf";
      const defaultName = `${conversationTitle.replace(/[^a-zA-Z0-9-_ ]/g, "")}.${ext}`;

      const outputPath = await save({
        defaultPath: defaultName,
        filters: [
          format === "markdown"
            ? { name: "Markdown", extensions: ["md"] }
            : { name: "PDF", extensions: ["pdf"] },
        ],
      });

      if (!outputPath) {
        setExporting(false);
        return;
      }

      if (format === "markdown") {
        await exportMarkdown(conversationId, outputPath);
      } else {
        await exportPdf(conversationId, outputPath);
      }

      setSuccess(true);
    } catch {
      setError("Export failed. Please try again.");
    } finally {
      setExporting(false);
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-surface rounded-lg border border-border p-6 w-full max-w-sm space-y-4">
        <h2 className="text-lg font-semibold text-text">
          Export Conversation
        </h2>

        {success ? (
          <div className="space-y-4">
            <p className="text-sm text-success">
              Exported successfully!
            </p>
            <button
              onClick={onClose}
              className="w-full px-4 py-2 text-sm bg-accent text-bg font-medium rounded-lg hover:bg-accent-hover transition-colors"
            >
              Done
            </button>
          </div>
        ) : (
          <>
            <div className="space-y-2">
              <p className="text-sm text-text-muted">Choose format:</p>
              <label
                className={`flex items-center gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${
                  format === "markdown"
                    ? "border-accent bg-accent/5"
                    : "border-border hover:border-border-hover"
                }`}
              >
                <input
                  type="radio"
                  name="format"
                  value="markdown"
                  checked={format === "markdown"}
                  onChange={() => setFormat("markdown")}
                  className="accent-accent"
                />
                <div>
                  <span className="text-sm font-medium text-text">
                    Markdown
                  </span>
                  <p className="text-xs text-text-dim">.md file</p>
                </div>
              </label>
              <label
                className={`flex items-center gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${
                  format === "pdf"
                    ? "border-accent bg-accent/5"
                    : "border-border hover:border-border-hover"
                }`}
              >
                <input
                  type="radio"
                  name="format"
                  value="pdf"
                  checked={format === "pdf"}
                  onChange={() => setFormat("pdf")}
                  className="accent-accent"
                />
                <div>
                  <span className="text-sm font-medium text-text">PDF</span>
                  <p className="text-xs text-text-dim">.pdf file</p>
                </div>
              </label>
            </div>

            {error && <p className="text-sm text-error">{error}</p>}

            <div className="flex gap-2">
              <button
                onClick={handleExport}
                disabled={exporting}
                className="flex-1 px-4 py-2 text-sm bg-accent text-bg font-medium rounded-lg hover:bg-accent-hover disabled:opacity-50 transition-colors"
              >
                {exporting ? "Exporting..." : "Export"}
              </button>
              <button
                onClick={onClose}
                disabled={exporting}
                className="px-4 py-2 text-sm text-text-muted hover:text-text rounded-lg hover:bg-surface-hover transition-colors"
              >
                Cancel
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
