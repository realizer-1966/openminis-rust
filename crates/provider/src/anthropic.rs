// Anthropic Claude 프로바이더
// 원본: provider/anthropic/ 디렉토리

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc;
use tracing::debug;

use crate::types::*;
use crate::traits::LlmProvider;

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com".into(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<AnthropicTool>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum AnthropicContent {
    Text { text: String },
    Image {
        source: ImageSource,
    },
    #[serde(rename = "tool_use")]
    ToolUse { id: String, name: String, input: serde_json::Value },
    #[serde(rename = "tool_result")]
    ToolResult { tool_use_id: String, content: String, #[serde(skip_serializing_if = "is_false")] is_error: bool },
}

fn is_false(b: &bool) -> bool { !*b }

#[derive(Debug, Serialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn provider_type(&self) -> &str { "anthropic" }

    async fn list_models(&self) -> Result<Vec<ModelEntry>> {
        // Anthropic는 모델 목록 API가 없으므로 하드코딩
        Ok(vec![
            ModelEntry {
                id: "claude-sonnet-4-5".into(),
                name: "Claude Sonnet 4.5".into(),
                provider: "anthropic".into(),
                context_window: Some(200_000),
                max_output_tokens: Some(16_384),
                supports_tools: true,
                supports_vision: true,
                supports_thinking: true,
            },
            ModelEntry {
                id: "claude-opus-4-1".into(),
                name: "Claude Opus 4.1".into(),
                provider: "anthropic".into(),
                context_window: Some(200_000),
                max_output_tokens: Some(32_768),
                supports_tools: true,
                supports_vision: true,
                supports_thinking: true,
            },
            ModelEntry {
                id: "claude-haiku-3-5".into(),
                name: "Claude Haiku 3.5".into(),
                provider: "anthropic".into(),
                context_window: Some(200_000),
                max_output_tokens: Some(8_192),
                supports_tools: true,
                supports_vision: true,
                supports_thinking: false,
            },
        ])
    }

    async fn stream_chat(
        &self,
        model: &str,
        messages: &[LlmMessage],
        tools: &[LlmToolDefinition],
        thinking: &ThinkingLevel,
        system_prompt: Option<&str>,
        tx: mpsc::Sender<StreamChunk>,
    ) -> Result<LlmUsage> {
        let anthropic_messages: Vec<AnthropicMessage> = messages
            .iter()
            .map(|m| AnthropicMessage {
                role: match m.role {
                    Role::User => "user".into(),
                    Role::Assistant => "assistant".into(),
                    Role::System => "user".into(), // 시스템은 별도 필드
                    Role::Tool => "user".into(),
                },
                content: m.content.iter().map(|p| match p {
                    ContentPart::Text { text } => AnthropicContent::Text { text: text.clone() },
                    ContentPart::Image { source } => AnthropicContent::Image { source: source.clone() },
                    ContentPart::ToolUse { id, name, input } => AnthropicContent::ToolUse {
                        id: id.clone(), name: name.clone(), input: input.clone(),
                    },
                    ContentPart::ToolResult { tool_use_id, content, is_error } => AnthropicContent::ToolResult {
                        tool_use_id: tool_use_id.clone(), content: content.clone(), is_error: *is_error,
                    },
                }).collect(),
            })
            .collect();

        let anthropic_tools: Vec<AnthropicTool> = tools
            .iter()
            .map(|t| AnthropicTool {
                name: t.name.clone(),
                description: t.description.clone(),
                input_schema: t.input_schema.clone(),
            })
            .collect();

        let mut request_body = json!({
            "model": model,
            "messages": anthropic_messages,
            "max_tokens": 16384,
            "stream": true,
        });

        if let Some(sys) = system_prompt {
            request_body["system"] = json!(sys);
        }

        if !anthropic_tools.is_empty() {
            request_body["tools"] = serde_json::to_value(&anthropic_tools)?;
        }

        // thinking 설정
        match thinking {
            ThinkingLevel::None => {}
            ThinkingLevel::Low => {
                request_body["thinking"] = json!({"type": "enabled", "budget_tokens": 5000});
            }
            ThinkingLevel::Medium => {
                request_body["thinking"] = json!({"type": "enabled", "budget_tokens": 10000});
            }
            ThinkingLevel::High => {
                request_body["thinking"] = json!({"type": "enabled", "budget_tokens": 20000});
            }
            ThinkingLevel::Max => {
                request_body["thinking"] = json!({"type": "enabled", "budget_tokens": 32000});
            }
        }

        let url = format!("{}/v1/messages", self.base_url);
        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error {}: {}", status, body);
        }

        // SSE 스트림 파싱
        let stream = response.bytes_stream();
        use tokio_stream::StreamExt;
        let mut stream = stream;
        let mut usage = LlmUsage::default();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            let chunk_bytes: &[u8] = &chunk;
            buffer.push_str(&String::from_utf8_lossy(chunk_bytes));

            // 줄 단위로 처리
            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer = buffer[pos + 1..].to_string();

                if line.is_empty() || line.starts_with(':') {
                    continue;
                }

                if let Some(rest) = line.strip_prefix("data: ") {
                    if rest == "[DONE]" {
                        let _ = tx.send(StreamChunk::Done).await;
                        continue;
                    }

                    if let Ok(event) = serde_json::from_str::<AnthropicSseEvent>(rest) {
                        match event.event_type.as_str() {
                            "content_block_delta" => {
                                if let Some(delta) = event.delta {
                                    match delta.delta_type.as_str() {
                                        "text_delta" => {
                                            if let Some(text) = delta.text {
                                                let _ = tx.send(StreamChunk::TextDelta { text }).await;
                                            }
                                        }
                                        "thinking_delta" => {
                                            if let Some(text) = delta.thinking {
                                                let _ = tx.send(StreamChunk::ThinkingDelta { text }).await;
                                            }
                                        }
                                        "input_json_delta" => {
                                            if let Some(partial) = delta.partial_json {
                                                // tool_use delta — id를 알 수 없으므로 빈 문자열
                                                let _ = tx.send(StreamChunk::ToolUseDelta {
                                                    id: String::new(),
                                                    partial_json: partial,
                                                }).await;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            "content_block_start" => {
                                if let Some(block) = event.content_block {
                                    if block.block_type == "tool_use" {
                                        let _ = tx.send(StreamChunk::ToolUseStart {
                                            id: block.id.unwrap_or_default(),
                                            name: block.name.unwrap_or_default(),
                                        }).await;
                                    }
                                }
                            }
                            "content_block_stop" => {
                                // 블록 종료 — tool_use인 경우 ToolUseStop
                            }
                            "message_delta" => {
                                if let Some(usage_data) = event.usage {
                                    if let Some(out) = usage_data.output_tokens {
                                        usage.output_tokens = out;
                                    }
                                }
                            }
                            "message_start" => {
                                if let Some(msg) = event.message {
                                    if let Some(u) = msg.usage {
                                        usage.input_tokens = u.input_tokens.unwrap_or(0);
                                        usage.cache_read_tokens = u.cache_read_tokens.unwrap_or(0);
                                        usage.cache_creation_tokens = u.cache_creation_tokens.unwrap_or(0);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(usage)
    }

    async fn chat(
        &self,
        model: &str,
        messages: &[LlmMessage],
        tools: &[LlmToolDefinition],
        thinking: &ThinkingLevel,
        system_prompt: Option<&str>,
    ) -> Result<(String, LlmUsage)> {
        let (tx, mut rx) = mpsc::channel(256);
        let provider_clone = AnthropicProvider {
            client: self.client.clone(),
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
        };
        let model = model.to_string();
        let messages = messages.to_vec();
        let tools = tools.to_vec();
        let thinking = thinking.clone();
        let system = system_prompt.map(|s| s.to_string());

        let handle = tokio::spawn(async move {
            provider_clone.stream_chat(&model, &messages, &tools, &thinking, system.as_deref(), tx).await
        });

        let mut full_text = String::new();
        while let Some(chunk) = rx.recv().await {
            if let StreamChunk::TextDelta { text } = chunk {
                full_text.push_str(&text);
            }
        }

        let usage = handle.await??;
        Ok((full_text, usage))
    }
}

#[derive(Debug, Deserialize)]
struct AnthropicSseEvent {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<AnthropicDelta>,
    content_block: Option<AnthropicContentBlock>,
    message: Option<AnthropicMessageStart>,
    usage: Option<AnthropicUsageDelta>,
}

#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    delta_type: String,
    text: Option<String>,
    thinking: Option<String>,
    partial_json: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicMessageStart {
    usage: Option<AnthropicUsageDelta>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsageDelta {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
    cache_read_tokens: Option<u32>,
    cache_creation_tokens: Option<u32>,
}