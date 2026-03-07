use std::path::PathBuf;
use std::sync::Arc;

use serde_json::{json, Value};
use tauri::Manager;
use tokio::sync::Mutex;

use crate::db::Database;
use crate::services::audio_player::AudioPlayerState;
use crate::services::audio_validation;

#[tauri::command]
pub async fn upload_audio(
    app: tauri::AppHandle,
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    session_id: String,
    file_path: String,
) -> Result<Value, String> {
    let source_path = PathBuf::from(&file_path);

    // Validate file exists and has supported format
    audio_validation::validate_file_exists(&source_path).map_err(|e| e.to_string())?;
    audio_validation::validate_format(&source_path).map_err(|e| e.to_string())?;

    // Check disk space in app data dir
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {}", e))?;
    audio_validation::check_disk_space(&app_data_dir).map_err(|e| e.to_string())?;

    // Generate conversation ID and destination path
    let conversation_id = uuid::Uuid::new_v4().to_string();
    let ext = source_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("wav");
    let audio_dir = app_data_dir.join("audio");

    // Ensure audio directory exists (defensive against deletion after app startup)
    tokio::fs::create_dir_all(&audio_dir)
        .await
        .map_err(|e| format!("failed to create audio directory: {}", e))?;

    let dest_path = audio_dir.join(format!("{}.{}", conversation_id, ext));

    // Copy audio file to app data directory (always creates new conversation, even for duplicates)
    tokio::fs::copy(&source_path, &dest_path)
        .await
        .map_err(|e| format!("failed to copy audio file: {}", e))?;

    // Get file duration using rodio (best-effort)
    let dest_clone = dest_path.clone();
    let duration = tokio::task::spawn_blocking(move || get_audio_duration(&dest_clone))
        .await
        .map_err(|e| format!("failed to get duration: {}", e))?;

    // Check for duration warning
    let warnings: Vec<String> = audio_validation::check_duration_warning(duration)
        .into_iter()
        .collect();

    // Determine next sequence number for this session
    let db = db.lock().await;
    let seq_row: Option<(i64,)> = sqlx::query_as(
        "SELECT COALESCE(MAX(sequence_number), 0) FROM conversations WHERE session_id = ?",
    )
    .bind(&session_id)
    .fetch_optional(db.pool())
    .await
    .map_err(|e| format!("database error: {}", e))?;

    let next_seq = seq_row.map(|(n,)| n + 1).unwrap_or(1);

    // Derive title from filename
    let title = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();

    let now = chrono::Utc::now().to_rfc3339();

    // Create conversation record
    sqlx::query(
        "INSERT INTO conversations (id, session_id, sequence_number, title, audio_file_path, duration_seconds, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&conversation_id)
    .bind(&session_id)
    .bind(next_seq)
    .bind(&title)
    .bind(dest_path.to_str().unwrap_or(""))
    .bind(duration)
    .bind("uploading")
    .bind(&now)
    .bind(&now)
    .execute(db.pool())
    .await
    .map_err(|e| format!("failed to create conversation: {}", e))?;

    let mut response = json!({
        "conversationId": conversation_id,
        "status": "uploading"
    });

    if !warnings.is_empty() {
        response["warnings"] = json!(warnings);
    }

    Ok(response)
}

/// Get audio file duration using rodio's decoder.
fn get_audio_duration(path: &PathBuf) -> f64 {
    use rodio::Source;
    use std::fs::File;
    use std::io::BufReader;

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return 0.0,
    };

    let reader = BufReader::new(file);
    match rodio::Decoder::new(reader) {
        Ok(decoder) => decoder
            .total_duration()
            .map(|d: std::time::Duration| d.as_secs_f64())
            .unwrap_or(0.0),
        Err(_) => 0.0,
    }
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
    let row: Option<(String, f64)> =
        sqlx::query_as("SELECT audio_file_path, duration_seconds FROM conversations WHERE id = ?")
            .bind(&conversation_id)
            .fetch_optional(db.pool())
            .await
            .map_err(|e| format!("database error: {}", e))?;

    let (audio_path, db_duration) = row.ok_or_else(|| {
        format!(
            "AudioFileNotFound: no conversation with id {}",
            conversation_id
        )
    })?;
    drop(db); // Release DB lock before blocking on audio

    let file_path = PathBuf::from(&audio_path);

    // Verify the file still exists on disk
    if !file_path.exists() {
        return Err(format!(
            "AudioFileNotFound: audio file missing from disk: {}",
            file_path.display()
        ));
    }

    let result = player
        .play(&file_path, seek_seconds)
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
pub async fn pause_audio(player: tauri::State<'_, AudioPlayerState>) -> Result<Value, String> {
    let result = player
        .pause()
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
    let result = player
        .get_position()
        .map_err(|e| format!("PlaybackError: {}", e))?;

    Ok(json!({
        "positionSeconds": result.position_seconds,
        "playing": result.playing
    }))
}
