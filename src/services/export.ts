import { invoke } from "@tauri-apps/api/core";

export async function exportMarkdown(
  conversationId: string,
  outputPath: string,
): Promise<{ exported: true; path: string }> {
  return invoke("export_markdown", { conversationId, outputPath });
}

export async function exportPdf(
  conversationId: string,
  outputPath: string,
): Promise<{ exported: true; path: string }> {
  return invoke("export_pdf", { conversationId, outputPath });
}
