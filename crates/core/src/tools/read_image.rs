// read_image 툴 — 이미지 파일을 읽어 시각적 분석용으로 반환
// 원본: ReadImageTool.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::Path;
use crate::tool_dispatch::{Tool, ToolDefinition, ToolResult};

pub struct ReadImageTool;

#[async_trait]
impl Tool for ReadImageTool {
    fn name(&self) -> &str { "read_image" }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "read_image".into(),
            description: "Read an image file for visual analysis. Supports PNG, JPEG, GIF, WEBP.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Linux path or minis:// URL" }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let path = params["path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'path'"))?;

        let path = if path.starts_with("minis://") {
            path.replace("minis://", "/var/minis/")
        } else {
            path.to_string()
        };

        let path = Path::new(&path);
        if !path.exists() {
            return Ok(ToolResult {
                tool_name: "read_image".into(),
                tool_id: String::new(),
                success: false,
                output: format!("Image not found: {}", path.display()),
                minis_urls: vec![],
            });
        }

        let metadata = std::fs::metadata(path)?;
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        Ok(ToolResult {
            tool_name: "read_image".into(),
            tool_id: String::new(),
            success: true,
            output: format!("Image loaded: {} ({} bytes, .{}) — ready for visual analysis",
                path.display(), metadata.len(), ext),
            minis_urls: vec![format!("minis://{}", 
                path.strip_prefix("/var/minis/").map(|p| p.display().to_string())
                    .unwrap_or_else(|_| path.display().to_string()))],
        })
    }
}