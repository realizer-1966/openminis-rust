// 예약 작업 저장소

use anyhow::Result;
use std::path::PathBuf;
use crate::task::ScheduledTask;

pub struct ScheduledTaskStore {
    dir: PathBuf,
}

impl ScheduledTaskStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn save(&self, task: &ScheduledTask) -> Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let path = self.dir.join(format!("{}.json", task.id));
        let json = serde_json::to_string_pretty(task)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn list(&self) -> Vec<ScheduledTask> {
        let mut tasks = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(task) = serde_json::from_str::<ScheduledTask>(&content) {
                            tasks.push(task);
                        }
                    }
                }
            }
        }
        tasks
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let path = self.dir.join(format!("{}.json", id));
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}
