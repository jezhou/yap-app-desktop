use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::db::Database;
use crate::services::conversations as conversations_service;

#[tauri::command]
pub async fn list_conversations(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    session_id: String,
) -> Result<Value, String> {
    let db = db.lock().await;
    let conversations = conversations_service::list_conversations(db.pool(), &session_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(conversations).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn get_conversation_detail(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    conversation_id: String,
) -> Result<Value, String> {
    let db = db.lock().await;
    let detail = conversations_service::get_conversation_detail(db.pool(), &conversation_id)
        .await
        .map_err(|e| e.to_string())?;

    match detail {
        Some(d) => Ok(json!({
            "conversation": serde_json::to_value(&d.conversation).map_err(|e| e.to_string())?,
            "transcription": d.transcription.as_ref().map(|t| serde_json::to_value(t).ok()).flatten(),
            "summary": d.summary.as_ref().map(|s| serde_json::to_value(s).ok()).flatten(),
            "speakerRoles": serde_json::to_value(&d.speaker_roles).map_err(|e| e.to_string())?
        })),
        None => Err(format!("conversation not found: {}", conversation_id)),
    }
}

#[tauri::command]
pub async fn rename_conversation(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    conversation_id: String,
    title: String,
) -> Result<Value, String> {
    let db = db.lock().await;
    let updated = conversations_service::rename_conversation(db.pool(), &conversation_id, &title)
        .await
        .map_err(|e| e.to_string())?;

    if !updated {
        return Err(format!("conversation not found: {}", conversation_id));
    }

    Ok(json!({ "updated": true }))
}

#[tauri::command]
pub async fn delete_conversation(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    conversation_id: String,
) -> Result<Value, String> {
    let db = db.lock().await;
    let audio_path = conversations_service::delete_conversation(db.pool(), &conversation_id)
        .await
        .map_err(|e| e.to_string())?;

    match audio_path {
        Some(path) => {
            // Best-effort delete of the audio file
            let _ = std::fs::remove_file(&path);
            Ok(json!({ "deleted": true }))
        }
        None => Err(format!("conversation not found: {}", conversation_id)),
    }
}

#[tauri::command]
pub async fn update_speaker_role(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    conversation_id: String,
    speaker_label: String,
    display_name: String,
) -> Result<Value, String> {
    let db = db.lock().await;
    conversations_service::upsert_speaker_role(
        db.pool(),
        &conversation_id,
        &speaker_label,
        &display_name,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(json!({ "updated": true }))
}
