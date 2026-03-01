CREATE TABLE IF NOT EXISTS speaker_roles (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    speaker_label TEXT NOT NULL,
    display_name TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_speaker_roles_conversation_id ON speaker_roles(conversation_id)