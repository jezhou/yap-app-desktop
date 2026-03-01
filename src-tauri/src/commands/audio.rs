use serde_json::{json, Value};

#[tauri::command]
pub async fn upload_audio(
    session_id: String,
    file_path: String,
) -> Result<Value, String> {
    // Stub: will be implemented with audio service
    Ok(json!({
        "conversationId": uuid::Uuid::new_v4().to_string(),
        "status": "uploading"
    }))
}

#[tauri::command]
pub async fn play_audio(
    conversation_id: String,
    seek_seconds: Option<f64>,
) -> Result<Value, String> {
    // Stub: will be implemented with audio player service
    Ok(json!({
        "playing": true,
        "durationSeconds": 0.0
    }))
}

#[tauri::command]
pub async fn pause_audio() -> Result<Value, String> {
    // Stub: will be implemented with audio player service
    Ok(json!({
        "playing": false,
        "positionSeconds": 0.0
    }))
}

#[tauri::command]
pub async fn get_audio_position() -> Result<Value, String> {
    // Stub: will be implemented with audio player service
    Ok(json!({
        "positionSeconds": 0.0,
        "playing": false
    }))
}
