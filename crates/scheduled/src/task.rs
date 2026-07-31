// 예약 작업 정의

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub time: String, // HH:MM
    pub prompt: String,
    pub label: Option<String>,
    pub repeat_mode: RepeatMode,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepeatMode {
    #[default]
    Once,
    Daily,
    Weekdays,
    Custom { days: Vec<String> },
}
