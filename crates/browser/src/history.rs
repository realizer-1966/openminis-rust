// 브라우저 히스토리 — 방문 기록 관리
// 원본: BrowserHistory.kt

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub visited_at: DateTime<Utc>,
}

pub struct BrowserHistory {
    entries: Vec<HistoryEntry>,
    max_entries: usize,
}

impl BrowserHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    pub fn add(&mut self, url: &str, title: &str) {
        self.entries.push(HistoryEntry {
            url: url.into(),
            title: title.into(),
            visited_at: Utc::now(),
        });
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    pub fn list(&self) -> &[HistoryEntry] {
        &self.entries
    }
}

impl Default for BrowserHistory {
    fn default() -> Self { Self::new(100) }
}
