pub mod commands;
pub mod db;
pub mod models;
pub mod services;

use std::sync::Arc;

use tauri::Manager;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            // Ensure the audio files directory exists
            let audio_dir = app_data_dir.join("audio");
            std::fs::create_dir_all(&audio_dir)
                .expect("failed to create audio directory");

            let rt = tokio::runtime::Runtime::new()
                .expect("failed to create tokio runtime");
            let database = rt
                .block_on(db::Database::init(app_data_dir))
                .expect("failed to initialize database");

            app.manage(Arc::new(Mutex::new(database)) as db::DbState);

            // Initialize audio player
            let audio_player: services::audio_player::AudioPlayerState =
                Arc::new(services::audio_player::AudioPlayer::new());
            app.manage(audio_player);

            // Initialize transcription state for cancellation tracking
            let transcription_state: commands::transcription::TranscriptionStateHandle =
                Arc::new(commands::transcription::TranscriptionState::new());
            app.manage(transcription_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Audio
            commands::audio::upload_audio,
            commands::audio::play_audio,
            commands::audio::pause_audio,
            commands::audio::get_audio_position,
            // Transcription
            commands::transcription::start_transcription,
            commands::transcription::cancel_transcription,
            // Sessions
            commands::sessions::create_session,
            commands::sessions::list_sessions,
            commands::sessions::update_session,
            commands::sessions::delete_session,
            // Conversations
            commands::conversations::list_conversations,
            commands::conversations::get_conversation_detail,
            commands::conversations::rename_conversation,
            commands::conversations::delete_conversation,
            commands::conversations::update_speaker_role,
            // Search
            commands::search::search_conversations,
            // Export
            commands::export::export_markdown,
            commands::export::export_pdf,
            // Settings
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::list_available_models,
            commands::settings::download_model,
            commands::settings::delete_model,
            commands::settings::get_diarization_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
