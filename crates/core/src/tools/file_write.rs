// file_write 툴 — 파일 쓰기 (생성/덮어쓰기)
// 원본: FileWriteTool.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::Path;
use crate::tool_dispatch::{Tool, ToolDefinition, ToolResult};

pub struct FileWriteTool;

#[async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str { "file_write" }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_write".into(),
            description: "Write content to a file. Creates the file if it doesn't exist, overwrites if it does.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute Linux path to write" },
                    "content": { "type": "string", "description": "The text content to write" },
                    "append": { "type": "boolean", "description": "If true, append to existing file" }
                },
                "required": ["path", "content"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let path = params["path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let content = params["content"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;
        let append = params["append"].as_bool().unwrap_or(false);
        let create_dirs = params["create_dirs"].as_bool().unwrap_or(true);

        let path = Path::new(path);

        // 부모 디렉토리 생성
        if create_dirs {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
        }

        if append {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            file.write_all(content.as_bytes())?;
        } else {
            std::fs::write(path, content)?;
        }

        let bytes = content.len();
        let minis_urls = if path.starts_with("/var/minis/") {
            let relative = path.strip_prefix("/var/minis/").unwrap();
            vec![format!("minis://{}", relative.display())]
        } else {
            vec![]
        };

        Ok(ToolResult {
            tool_name: "file_write".into(),
            tool_id: String::new(),
            success: true,
            output: format!("Wrote {} bytes to {}", bytes, path.display()),
            minis_urls,
        })
    }
}