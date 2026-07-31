// file_edit 툴 — 기존 파일의 일부 교체 (old_string → new_string)
// 원본: FileEditTool.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::Path;
use crate::tool_dispatch::{Tool, ToolDefinition, ToolResult};

pub struct FileEditTool;

#[async_trait]
impl Tool for FileEditTool {
    fn name(&self) -> &str { "file_edit" }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_edit".into(),
            description: "Make targeted edits to an existing file using exact string replacement.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute Linux path to edit" },
                    "old_string": { "type": "string", "description": "The exact text to find" },
                    "new_string": { "type": "string", "description": "The replacement text" },
                    "replace_all": { "type": "boolean", "description": "Replace ALL occurrences" }
                },
                "required": ["path", "old_string", "new_string"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let path = params["path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'path'"))?;
        let old_string = params["old_string"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'old_string'"))?;
        let new_string = params["new_string"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'new_string'"))?;
        let replace_all = params["replace_all"].as_bool().unwrap_or(false);

        let path = Path::new(path);
        let content = std::fs::read_to_string(path)?;

        let new_content = if replace_all {
            content.replace(old_string, new_string)
        } else {
            // 정확히 한 번만 교체
            let count = content.matches(old_string).count();
            if count == 0 {
                return Ok(ToolResult {
                    tool_name: "file_edit".into(),
                    tool_id: String::new(),
                    success: false,
                    output: format!("old_string not found in {}", path.display()),
                    minis_urls: vec![],
                });
            }
            if count > 1 {
                return Ok(ToolResult {
                    tool_name: "file_edit".into(),
                    tool_id: String::new(),
                    success: false,
                    output: format!("old_string found {} times in {} — use replace_all=true", count, path.display()),
                    minis_urls: vec![],
                });
            }
            content.replacen(old_string, new_string, 1)
        };

        std::fs::write(path, &new_content)?;

        Ok(ToolResult {
            tool_name: "file_edit".into(),
            tool_id: String::new(),
            success: true,
            output: format!("Edited {}", path.display()),
            minis_urls: vec![],
        })
    }
}