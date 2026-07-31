// shell_execute 툴 — Linux 셸 명령 실행 (PersistentShell経由)
// 원본: ShellExecutor.kt, PersistentShell.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use crate::tool_dispatch::{Tool, ToolDefinition, ToolResult};
use minis_sandbox::shell::{PersistentShell, ShellConfig};

pub struct ShellExecuteTool {
    shell: Arc<PersistentShell>,
}

impl ShellExecuteTool {
    pub fn new(shell: Arc<PersistentShell>) -> Self {
        Self { shell }
    }

    /// 기본 셸으로 생성 (호스트 환경)
    pub fn default() -> Self {
        Self::new(Arc::new(PersistentShell::new()))
    }
}

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
        let timeout_secs = params["timeout"].as_u64().unwrap_or(900);

        let result = self.shell.execute(command, Duration::from_secs(timeout_secs)).await?;

        // 출력 조합 — stdout + stderr (에러시 stderr 포함)
        let mut output = result.stdout;
        if !result.stderr.is_empty() {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(&result.stderr);
        }

        // 타임아웃 표시
        if result.timed_out {
            output.push_str(&format!("\n[TIMEOUT] Command timed out after {}s", timeout_secs));
        }

        // minis:// URL이 있으면 추가
        if !result.new_minis_urls.is_empty() {
            output.push_str("\n[minis] New files:\n");
            for url in &result.new_minis_urls {
                output.push_str(&format!("  {}\n", url));
            }
        }

        Ok(ToolResult {
            tool_name: "shell_execute".into(),
            tool_id: String::new(),
            success: result.exit_code == 0,
            output,
            minis_urls: result.new_minis_urls,
        })
    }
}