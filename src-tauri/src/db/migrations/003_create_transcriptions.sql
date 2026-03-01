CREATE TABLE IF NOT EXISTS transcriptions (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL UNIQUE REFERENCES conversations(id) ON DELETE CASCADE,
    full_text TEXT NOT NULL,
    segments TEXT NOT NULL,
    created_at TEXT NOT NULL
)