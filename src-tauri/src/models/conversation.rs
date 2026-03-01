use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub session_id: String,
    pub sequence_number: i64,
    pub title: String,
    pub audio_file_path: String,
    pub duration_seconds: f64,
    pub status: ConversationStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConversationStatus {
    Uploading,
    Analyzing,
    Completed,
    Error,
}

impl std::fmt::Display for ConversationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uploading => write!(f, "uploading"),
            Self::Analyzing => write!(f, "analyzing"),
            Self::Completed => write!(f, "completed"),
            Self::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for ConversationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "uploading" => Ok(Self::Uploading),
            "analyzing" => Ok(Self::Analyzing),
            "completed" => Ok(Self::Completed),
            "error" => Ok(Self::Error),
            _ => Err(format!("invalid conversation status: {}", s)),
        }
    }
}
