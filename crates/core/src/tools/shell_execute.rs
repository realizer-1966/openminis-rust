// shell_execute 툴 — Linux 셸 명령 실행 (PRoot 환경)
// 원본: ShellExecutor.kt, PersistentShell.kt
// 실제 구현은 sandbox crate의 PersistentShell을 통해 실행

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::tool_dispatch::{Tool, ToolDefinition, ToolResult};

pub struct ShellExecuteTool;

#[async_trait]
impl Tool for ShellExecuteTool {
    fn name(&self) -> &str { "shell_execute" }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "shell_execute".into(),
            description: "Execute a command in the Linux sandbox (Alpine Linux via PRoot).".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "The shell command to execute" },
                    "timeout": { "type": "integer", "description": "Timeout in seconds (default 900)" }
                },
                "required": ["command"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let command = params["command"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'command'"))?;
        let _timeout = params["timeout"].as_u64().unwrap_or(900);

        // TODO: sandbox crate의 PersistentShell을 통해 실행
        // 현재는 placeholder
        Ok(ToolResult {
            tool_name: "shell_execute".into(),
            tool_id: String::new(),
            success: true,
            output: format!("[shell_execute placeholder] command: {}", command),
            minis_urls: vec![],
        })
    }
}