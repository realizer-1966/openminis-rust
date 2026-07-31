// OffloadHandler trait — 각 Android 기능이 구현

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffloadResult {
    pub success: bool,
    pub data: Value,
    pub error: Option<String>,
}

#[async_trait]
pub trait OffloadHandler: Send + Sync {
    /// 핸들러 이름 (e.g. "calendar", "contacts", "alarm")
    fn name(&self) -> &str;

    /// 요청 실행 — args는 JSON
    async fn execute(&self, args: Value) -> Result<OffloadResult>;
}
