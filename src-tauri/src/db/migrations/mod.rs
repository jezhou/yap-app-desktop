/// Returns all migrations in order as (name, sql) tuples.
pub fn all() -> Vec<(&'static str, &'static str)> {
    vec![
        ("001_create_sessions", include_str!("001_create_sessions.sql")),
        ("002_create_conversations", include_str!("002_create_conversations.sql")),
        ("003_create_transcriptions", include_str!("003_create_transcriptions.sql")),
        ("004_create_summaries", include_str!("004_create_summaries.sql")),
        ("005_create_speaker_roles", include_str!("005_create_speaker_roles.sql")),
        ("006_create_settings", include_str!("006_create_settings.sql")),
        ("007_create_fts", include_str!("007_create_fts.sql")),
    ]
}
