// 툴 디스패치 — LLM이 요청한 툴을 실행하고 결과 반환
// 원본: AgentTools.kt, tools/ 디렉토리

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 툴 정의 — LLM에게 전달되는 툴 스키마
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value, // JSON Schema
}

/// 툴 실행 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub tool_id: String,
    pub success: bool,
    pub output: String,
    /// minis:// URL이 생성된 경우
    pub minis_urls: Vec<String>,
}

/// 툴 trait — 각 툴이 구현
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn definition(&self) -> ToolDefinition;
    async fn execute(&self, params: Value) -> Result<ToolResult>;
}

/// 툴 디스패처 — 등록된 툴을 이름으로 찾아 실행
pub struct ToolDispatcher {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolDispatcher {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.iter().map(|t| t.definition()).collect()
    }

    pub async fn execute(&self, name: &str, params: Value) -> Result<ToolResult> {
        let tool = self.tools.iter().find(|t| t.name() == name)
            .ok_or_else(|| anyhow::anyhow!("Unknown tool: {}", name))?;
        tool.execute(params).await
    }
}

impl Default for ToolDispatcher {
    fn default() -> Self {
        Self::new()
    }
}