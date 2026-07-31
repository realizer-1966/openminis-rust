// 툴 디스패치 — LLM이 요청한 툴을 실행하고 결과 반환
// 원본: AgentTools.kt, tools/ 디렉토리

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use minis_provider::LlmToolDefinition;

/// 툴 정의 — LLM에게 전달되는 툴 스키마
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value, // JSON Schema
}

impl From<ToolDefinition> for LlmToolDefinition {
    fn from(def: ToolDefinition) -> Self {
        LlmToolDefinition {
            name: def.name,
            description: def.description,
            input_schema: def.parameters,
        }
    }
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

    /// LLM 프로바이더에 전달할 수 있는 툴 정의 목록
    pub fn llm_definitions(&self) -> Vec<LlmToolDefinition> {
        self.tools.iter().map(|t| t.definition().into()).collect()
    }

    pub async fn execute(&self, name: &str, params: Value) -> Result<ToolResult> {
        let tool = self.tools.iter().find(|t| t.name() == name)
            .ok_or_else(|| anyhow::anyhow!("Unknown tool: {}", name))?;
        tool.execute(params).await
    }

    /// 여러 툴을 동시에 실행 (concurrent_tools가 true일 때)
    pub async fn execute_batch(&self, calls: &[(String, String, Value)]) -> Vec<ToolResult> {
        let mut results = Vec::with_capacity(calls.len());
        for (name, id, params) in calls {
            let result = match self.execute(name, params.clone()).await {
                Ok(mut r) => {
                    r.tool_id = id.clone();
                    r
                }
                Err(e) => ToolResult {
                    tool_name: name.clone(),
                    tool_id: id.clone(),
                    success: false,
                    output: format!("Error: {}", e),
                    minis_urls: vec![],
                }
            };
            results.push(result);
        }
        results
    }
}

impl Default for ToolDispatcher {
    fn default() -> Self {
        Self::new()
    }
}