CREATE TABLE IF NOT EXISTS fts_rowid_map (
    conversation_id TEXT PRIMARY KEY,
    fts_rowid INTEGER NOT NULL
)