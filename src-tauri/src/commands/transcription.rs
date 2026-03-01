use serde_json::{json, Value};

#[tauri::command]
pub async fn start_transcription(
    conversation_id: String,
) -> Result<Value, String> {
    // Stub: will be implemented with transcription service + sherpa-rs
    Ok(json!({
        "status": "analyzing"
    }))
}

#[tauri::command]
pub async fn cancel_transcription(
    conversation_id: String,
) -> Result<Value, String> {
    // Stub: will be implemented with transcription service
    Ok(json!({
        "status": "error",
        "reason": "cancelled"
    }))
}
