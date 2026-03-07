use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::db::Database;
use crate::services::settings as settings_service;

#[tauri::command]
pub async fn get_settings(db: tauri::State<'_, Arc<Mutex<Database>>>) -> Result<Value, String> {
    let db = db.lock().await;
    let settings = settings_service::get_all_settings(db.pool())
        .await
        .map_err(|e| e.to_string())?;

    // Return as array of {key, value} objects to match TypeScript Settings[] type
    Ok(serde_json::to_value(settings).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn update_settings(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    key: String,
    value: Value,
) -> Result<Value, String> {
    let db = db.lock().await;
    // Serialize value to string for storage
    let value_str = match &value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };

    settings_service::upsert_setting(db.pool(), &key, &value_str)
        .await
        .map_err(|e| e.to_string())?;

    Ok(json!({ "updated": true }))
}

#[tauri::command]
pub async fn validate_api_key(api_key: String) -> Result<Value, String> {
    if api_key.trim().is_empty() {
        return Ok(json!({ "valid": false, "error": "API key cannot be empty" }));
    }

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.deepgram.com/v1/projects")
        .header("Authorization", format!("Token {}", api_key.trim()))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        Ok(json!({ "valid": true }))
    } else if response.status().as_u16() == 401 || response.status().as_u16() == 403 {
        Ok(json!({ "valid": false, "error": "Invalid API key" }))
    } else {
        Ok(json!({ "valid": false, "error": format!("Unexpected response: HTTP {}", response.status()) }))
    }
}
