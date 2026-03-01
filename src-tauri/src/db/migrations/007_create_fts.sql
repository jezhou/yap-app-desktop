CREATE VIRTUAL TABLE IF NOT EXISTS transcription_fts USING fts5(
    session_title,
    conversation_title,
    full_text,
    summary_content,
    content='',
    contentless_delete=1
)