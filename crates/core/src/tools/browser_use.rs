// browser_use 툴 — 브라우저 자동화 (WebView 제어는 offload経由)
// 원본: BrowserUseTool.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::tool_dispatch::{Tool, ToolDefinition, ToolResult};

pub struct BrowserUseTool;

#[async_trait]
impl Tool for BrowserUseTool {
    fn name(&self) -> &str { "browser_use" }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "browser_use".into(),
            description: "Control a web browser — navigate, click, type, scroll, screenshot, etc.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["navigate", "screenshot", "click", "type", "scroll", "get_text", "get_readable", "find_elements", "hover", "execute_js", "fetch"] },
                    "url": { "type": "string" },
                    "selector": { "type": "string" },
                    "text": { "type": "string" },
                    "script": { "type": "string" }
                },
                "required": ["action"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let action = params["action"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'action'"))?;
        // TODO: browser crate + offload経由でWebView制御
        Ok(ToolResult {
            tool_name: "browser_use".into(),
            tool_id: String::new(),
            success: true,
            output: format!("[browser_use placeholder] action: {}", action),
            minis_urls: vec![],
        })
    }
}