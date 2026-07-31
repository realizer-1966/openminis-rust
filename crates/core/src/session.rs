// 세션 — 채팅 세션 라이프사이클 관리
// 원본: ChatSessionEntity.kt, SessionManager

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub provider_id: String,
    pub model_id: String,
}

impl Session {
    pub fn new(provider_id: &str, model_id: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: "New Chat".into(),
            created_at: now,
            updated_at: now,
            provider_id: provider_id.into(),
            model_id: model_id.into(),
        }
    }
}