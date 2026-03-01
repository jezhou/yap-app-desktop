use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub id: String,
    pub conversation_id: String,
    pub content: String,
    pub created_at: String,
}
