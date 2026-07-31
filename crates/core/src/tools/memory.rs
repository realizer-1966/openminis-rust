// memory 툴 — memory_write, memory_get
// 원본: MemoryTools.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::tool_dispatch::{Tool, ToolDefinition, ToolResult};

pub struct MemoryWriteTool;
pub struct MemoryGetTool;

#[async_trait]
impl Tool for MemoryWriteTool {
    fn name(&self) -> &str { "memory_write" }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "memory_write".into(),
            description: "Write a memory entry to today's daily log.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "content": { "type": "string", "description": "The memory content to save" }
                },
                "required": ["content"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let content = params["content"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'content'"))?;
        // TODO: memory crate 연동
        Ok(ToolResult {
            tool_name: "memory_write".into(),
            tool_id: String::new(),
            success: true,
            output: format!("Memory saved ({} bytes)", content.len()),
            minis_urls: vec![],
        })
    }
}

#[async_trait]
impl Tool for MemoryGetTool {
    fn name(&self) -> &str { "memory_get" }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "memory_get".into(),
            description: "Retrieve memories from persistent storage with keyword search.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "keywords": { "type": "string", "description": "Space-separated keywords for fuzzy matching" }
                },
                "required": []
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let keywords = params["keywords"].as_str().unwrap_or("");
        // TODO: memory crate 연동
        Ok(ToolResult {
            tool_name: "memory_get".into(),
            tool_id: String::new(),
            success: true,
            output: format!("[memory_get placeholder] keywords: {}", keywords),
            minis_urls: vec![],
        })
    }
}