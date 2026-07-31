// 메모리 저장소

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub tags: Vec<String>,
}

pub struct MemoryStore {
    memory_dir: PathBuf,
}

impl MemoryStore {
    pub fn new(memory_dir: PathBuf) -> Self {
        Self { memory_dir }
    }

    /// 오늘의 일일 로그에 메모리 추가
    pub fn write(&self, content: &str) -> Result<()> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let log_path = self.memory_dir.join(format!("{}.md", today));
        
        // 디렉토리 생성
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let entry = format!("\n<!-- {} -->\n{}\n", timestamp, content);

        // 추가 모드로 작성
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        file.write_all(entry.as_bytes())?;

        Ok(())
    }

    /// 특정 날짜의 로그 읽기
    pub fn read_day(&self, date: &str) -> Result<Option<String>> {
        let path = self.memory_dir.join(format!("{}.md", date));
        if path.exists() {
            Ok(Some(std::fs::read_to_string(path)?))
        } else {
            Ok(None)
        }
    }

    /// 모든 메모리 파일 목록
    pub fn list_days(&self) -> Vec<String> {
        let mut days = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.memory_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".md") {
                        days.push(name.trim_end_matches(".md").to_string());
                    }
                }
            }
        }
        days.sort();
        days.reverse();
        days
    }
}
