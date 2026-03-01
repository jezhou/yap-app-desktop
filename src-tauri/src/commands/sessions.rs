use serde_json::{json, Value};

#[tauri::command]
pub async fn create_session(
    title: String,
    description: Option<String>,
) -> Result<Value, String> {
    // Stub: will be implemented with session CRUD service
    Ok(json!({
        "sessionId": uuid::Uuid::new_v4().to_string()
    }))
}

#[tauri::command]
pub async fn list_sessions(
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Value, String> {
    // Stub: will be implemented with session CRUD service
    Ok(json!([]))
}

#[tauri::command]
pub async fn update_session(
    session_id: String,
    title: Option<String>,
    description: Option<String>,
) -> Result<Value, String> {
    // Stub: will be implemented with session CRUD service
    Ok(json!({
        "updated": true
    }))
}

#[tauri::command]
pub async fn delete_session(
    session_id: String,
) -> Result<Value, String> {
    // Stub: will be implemented with session CRUD service
    Ok(json!({
        "deleted": true
    }))
}
