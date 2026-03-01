import { invoke } from "@tauri-apps/api/core";
import { writeFile } from "@tauri-apps/plugin-fs";

export async function exportMarkdown(
  conversationId: string,
  outputPath: string,
): Promise<{ exported: true; path: string }> {
  return invoke("export_markdown", { conversationId, outputPath });
}

export async function exportPdf(
  conversationId: string,
  outputPath: string,
): Promise<void> {
  // Get structured data from backend
  const result = await invoke<{ exported: true; path: string; data: unknown }>(
    "export_pdf",
    { conversationId, outputPath },
  );

  // Lazy-load pdfmake
  const pdfMakeModule = await import("pdfmake/build/pdfmake");
  const pdfMake = pdfMakeModule.default ?? pdfMakeModule;

  // Generate PDF from the document definition provided by backend
  const docDefinition = result.data as Parameters<typeof pdfMake.createPdf>[0];

  const pdfDoc = pdfMake.createPdf(docDefinition);

  // Get PDF as buffer and write to file
  const buffer = await pdfDoc.getBuffer();
  await writeFile(outputPath, new Uint8Array(buffer));
}
