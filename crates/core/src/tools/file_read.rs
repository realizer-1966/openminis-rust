// file_read 툴 — 파일 읽기
// 원본: FileReadTool.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::Path;
use crate::tool_dispatch::{Tool, ToolDefinition, ToolResult};

pub struct FileReadTool;

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str { "file_read" }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_read".into(),
            description: "Read a file from the Linux filesystem.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute Linux path to read" },
                    "offset": { "type": "integer", "description": "1-based line number to start from" },
                    "lines": { "type": "integer", "description": "Maximum number of lines to return" }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let path = params["path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let path = Path::new(path);

        if !path.exists() {
            return Ok(ToolResult {
                tool_name: "file_read".into(),
                tool_id: String::new(),
                success: false,
                output: format!("File not found: {}", path.display()),
                minis_urls: vec![],
            });
        }

        // 바이너리 파일 감지 (간단히 첫 바이트 확인)
        let metadata = std::fs::metadata(path)?;
        let content = if metadata.len() > 1_000_000 {
            format!("[File too large: {} bytes]", metadata.len())
        } else {
            std::fs::read_to_string(path)
                .unwrap_or_else(|_| "[Binary file or invalid UTF-8]".into())
        };

        // minis:// URL 생성 (경로가 /var/minis/ 하위인 경우)
        let minis_urls = if path.starts_with("/var/minis/") {
            let relative = path.strip_prefix("/var/minis/").unwrap();
            vec![format!("minis://{}", relative.display())]
        } else {
            vec![]
        };

        Ok(ToolResult {
            tool_name: "file_read".into(),
            tool_id: String::new(),
            success: true,
            output: content,
            minis_urls,
        })
    }
}