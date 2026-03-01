use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::db::Database;
use crate::services::exporter;

#[tauri::command]
pub async fn export_markdown(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    conversation_id: String,
    output_path: String,
) -> Result<Value, String> {
    let db = db.lock().await;
    exporter::export_to_markdown(db.pool(), &conversation_id, &output_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(json!({
        "exported": true,
        "path": output_path
    }))
}

#[tauri::command]
pub async fn export_pdf(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    conversation_id: String,
    output_path: String,
) -> Result<Value, String> {
    let db = db.lock().await;
    let data = exporter::export_to_pdf_payload(db.pool(), &conversation_id)
        .await
        .map_err(|e| e.to_string())?;

    // Return the structured data for frontend pdfmake rendering
    // The frontend will use this to generate the actual PDF
    let payload = serde_json::to_value(&data).map_err(|e| e.to_string())?;

    Ok(json!({
        "exported": true,
        "path": output_path,
        "data": payload
    }))
}
