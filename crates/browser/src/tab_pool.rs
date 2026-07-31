// 브라우저 탭 풀 — 최대 3개 탭 관리
// 원본: BrowserTabPool.kt

use std::collections::HashMap;

pub struct TabPool {
    tabs: HashMap<i32, TabInfo>,
    next_id: i32,
    max_tabs: usize,
}

#[derive(Debug, Clone)]
pub struct TabInfo {
    pub id: i32,
    pub url: String,
    pub title: String,
}

impl TabPool {
    pub fn new(max_tabs: usize) -> Self {
        Self {
            tabs: HashMap::new(),
            next_id: 0,
            max_tabs,
        }
    }

    pub fn create_tab(&mut self, url: &str) -> Option<i32> {
        if self.tabs.len() >= self.max_tabs {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.tabs.insert(id, TabInfo {
            id,
            url: url.into(),
            title: String::new(),
        });
        Some(id)
    }

    pub fn close_tab(&mut self, id: i32) {
        self.tabs.remove(&id);
    }

    pub fn get_tab(&self, id: i32) -> Option<&TabInfo> {
        self.tabs.get(&id)
    }

    pub fn list_tabs(&self) -> Vec<&TabInfo> {
        self.tabs.values().collect()
    }
}

impl Default for TabPool {
    fn default() -> Self { Self::new(3) }
}
