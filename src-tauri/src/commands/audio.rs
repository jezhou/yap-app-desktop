use std::path::PathBuf;
use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::db::Database;
use crate::services::audio_player::{AudioPlayer, AudioPlayerState};

#[tauri::command]
pub async fn upload_audio(
    session_id: String,
    file_path: String,
) -> Result<Value, String> {
    // Stub: will be fully implemented with audio service (T013)
    Ok(json!({
        "conversationId": uuid::Uuid::new_v4().to_string(),
        "status": "uploading"
    }))
}

#[tauri::command]
pub async fn play_audio(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    player: tauri::State<'_, AudioPlayerState>,
    conversation_id: String,
    seek_seconds: Option<f64>,
) -> Result<Value, String> {
    // Look up the audio file path from the conversation record
    let db = db.lock().await;
    let row: Option<(String, f64)> = sqlx::query_as(
        "SELECT audio_file_path, duration_seconds FROM conversations WHERE id = ?",
    )
    .bind(&conversation_id)
    .fetch_optional(db.pool())
    .await
    .map_err(|e| format!("database error: {}", e))?;

    let (audio_path, db_duration) = row.ok_or_else(|| {
        format!("AudioFileNotFound: no conversation with id {}", conversation_id)
    })?;
    drop(db); // Release DB lock before blocking on audio

    let file_path = PathBuf::from(&audio_path);
    let result = player.play(&file_path, seek_seconds)
        .map_err(|e| format!("PlaybackError: {}", e))?;

    // Use DB duration if decoder couldn't determine it
    let duration = if result.duration_seconds > 0.0 {
        result.duration_seconds
    } else {
        db_duration
    };

    Ok(json!({
        "playing": result.playing,
        "durationSeconds": duration
    }))
}

#[tauri::command]
pub async fn pause_audio(
    player: tauri::State<'_, AudioPlayerState>,
) -> Result<Value, String> {
    let result = player.pause()
        .map_err(|e| format!("PlaybackError: {}", e))?;

    Ok(json!({
        "playing": result.playing,
        "positionSeconds": result.position_seconds
    }))
}

#[tauri::command]
pub async fn get_audio_position(
    player: tauri::State<'_, AudioPlayerState>,
) -> Result<Value, String> {
    let result = player.get_position()
        .map_err(|e| format!("PlaybackError: {}", e))?;

    Ok(json!({
        "positionSeconds": result.position_seconds,
        "playing": result.playing
    }))
}
