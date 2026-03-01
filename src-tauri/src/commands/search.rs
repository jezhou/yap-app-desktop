use serde_json::{json, Value};

#[tauri::command]
pub async fn search_conversations(
    query: String,
    limit: Option<i64>,
) -> Result<Value, String> {
    // Stub: will be implemented with FTS5 search service
    Ok(json!([]))
}
