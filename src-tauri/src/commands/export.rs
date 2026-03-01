use serde_json::{json, Value};

#[tauri::command]
pub async fn export_markdown(
    conversation_id: String,
    output_path: String,
) -> Result<Value, String> {
    // Stub: will be implemented with exporter service
    Ok(json!({
        "exported": true,
        "path": output_path
    }))
}

#[tauri::command]
pub async fn export_pdf(
    conversation_id: String,
    output_path: String,
) -> Result<Value, String> {
    // Stub: will be implemented with exporter service
    Ok(json!({
        "exported": true,
        "path": output_path
    }))
}
