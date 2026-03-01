use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcription {
    pub id: String,
    pub conversation_id: String,
    pub full_text: String,
    pub segments: Vec<TranscriptionSegment>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub speaker: String,
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerRole {
    pub id: String,
    pub conversation_id: String,
    pub speaker_label: String,
    pub display_name: String,
}
