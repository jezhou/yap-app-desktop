use serde_json::{json, Value};

#[tauri::command]
pub async fn list_conversations(
    session_id: String,
) -> Result<Value, String> {
    // Stub: will be implemented with conversation CRUD service
    Ok(json!([]))
}

#[tauri::command]
pub async fn get_conversation_detail(
    conversation_id: String,
) -> Result<Value, String> {
    // Stub: will be implemented with conversation CRUD service
    Ok(json!({
        "conversation": null,
        "transcription": null,
        "summary": null,
        "speakerRoles": []
    }))
}

#[tauri::command]
pub async fn rename_conversation(
    conversation_id: String,
    title: String,
) -> Result<Value, String> {
    // Stub: will be implemented with conversation CRUD service
    Ok(json!({
        "updated": true
    }))
}

#[tauri::command]
pub async fn delete_conversation(
    conversation_id: String,
) -> Result<Value, String> {
    // Stub: will be implemented with conversation CRUD service
    Ok(json!({
        "deleted": true
    }))
}

#[tauri::command]
pub async fn update_speaker_role(
    conversation_id: String,
    speaker_label: String,
    display_name: String,
) -> Result<Value, String> {
    // Stub: will be implemented with conversation CRUD service
    Ok(json!({
        "updated": true
    }))
}
