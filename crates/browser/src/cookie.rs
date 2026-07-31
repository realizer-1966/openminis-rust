// 쿠키 관리 — 세션 쿠키 저장/복원
// 원본: CookieAuditLogger.kt, CookieBackupStore.swift

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub expires: Option<u64>,
}

pub struct CookieStore {
    cookies: HashMap<String, Vec<Cookie>>,
}

impl CookieStore {
    pub fn new() -> Self {
        Self { cookies: HashMap::new() }
    }

    pub fn save(&mut self, domain: &str, cookies: Vec<Cookie>) {
        self.cookies.insert(domain.into(), cookies);
    }

    pub fn load(&self, domain: &str) -> Vec<Cookie> {
        self.cookies.get(domain).cloned().unwrap_or_default()
    }

    pub fn clear_domain(&mut self, domain: &str) {
        self.cookies.remove(domain);
    }
}

impl Default for CookieStore {
    fn default() -> Self { Self::new() }
}
