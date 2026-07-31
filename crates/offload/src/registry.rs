// Offload 레지스트리 — 핸들러 등록/조회/실행

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use crate::trait_def::{OffloadHandler, OffloadResult};

pub struct OffloadRegistry {
    handlers: HashMap<String, Arc<dyn OffloadHandler>>,
}

impl OffloadRegistry {
    pub fn new() -> Self {
        Self { handlers: HashMap::new() }
    }

    pub fn register(&mut self, handler: Arc<dyn OffloadHandler>) {
        self.handlers.insert(handler.name().to_string(), handler);
    }

    pub async fn execute(&self, name: &str, args: Value) -> Result<OffloadResult> {
        let handler = self.handlers.get(name)
            .ok_or_else(|| anyhow::anyhow!("Unknown offload handler: {}", name))?;
        handler.execute(args).await
    }

    pub fn list_handlers(&self) -> Vec<&str> {
        self.handlers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for OffloadRegistry {
    fn default() -> Self { Self::new() }
}
