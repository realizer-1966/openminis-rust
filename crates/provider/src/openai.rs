// OpenAI 호환 프로바이더 (OpenAI, OpenRouter, xAI, Antigravity 등)
// 원본: provider/openai/, provider/openrouter/, provider/xai/, provider/antigravity/

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc;
use tracing::debug;

use crate::types::*;
use crate::traits::LlmProvider;

pub struct OpenAiCompatibleProvider {
    client: Client,
    api_key: String,
    base_url: String,
    provider_name: String,
}

impl OpenAiCompatibleProvider {
    pub fn new(provider_name: &str, api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
            provider_name: provider_name.into(),
        }
    }

    pub fn openai(api_key: String) -> Self {
        Self::new("openai", api_key, "https://api.openai.com/v1".into())
    }

    pub fn openrouter(api_key: String) -> Self {
        Self::new("openrouter", api_key, "https://openrouter.ai/api/v1".into())
    }

    pub fn xai(api_key: String) -> Self {
        Self::new("xai", api_key, "https://api.x.ai/v1".into())
    }
}

#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<OpenAiTool>,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAiTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAiFunction,
}

#[derive(Debug, Serialize)]
struct OpenAiFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    fn provider_type(&self) -> &str { &self.provider_name }

    async fn list_models(&self) -> Result<Vec<ModelEntry>> {
        let url = format!("{}/models", self.base_url);
        let response = self.client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to list models: {}", response.status());
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<OpenAiModel>,
        }
        #[derive(Deserialize)]
        struct OpenAiModel {
            id: String,
        }

        let resp: ModelsResponse = response.json().await?;
        Ok(resp.data.into_iter().map(|m| {
            let id = m.id;
            ModelEntry {
                id: id.clone(),
                name: id,
                provider: self.provider_name.clone(),
                context_window: None,
                max_output_tokens: None,
                supports_tools: true,
                supports_vision: false,
                supports_thinking: false,
            }
        }).collect())
    }

    async fn stream_chat(
        &self,
        model: &str,
        messages: &[LlmMessage],
        tools: &[LlmToolDefinition],
        _thinking: &ThinkingLevel,
        system_prompt: Option<&str>,
        tx: mpsc::Sender<StreamChunk>,
    ) -> Result<LlmUsage> {
        let mut openai_messages: Vec<OpenAiMessage> = Vec::new();

        if let Some(sys) = system_prompt {
            openai_messages.push(OpenAiMessage {
                role: "system".into(),
                content: sys.into(),
            });
        }

        for msg in messages {
            let text: String = msg.content.iter()
                .filter_map(|p| match p {
                    ContentPart::Text { text } => Some(text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");

            openai_messages.push(OpenAiMessage {
                role: match msg.role {
                    Role::User => "user".into(),
                    Role::Assistant => "assistant".into(),
                    Role::System => "system".into(),
                    Role::Tool => "tool".into(),
                },
                content: text,
            });
        }

        let openai_tools: Vec<OpenAiTool> = tools
            .iter()
            .map(|t| OpenAiTool {
                tool_type: "function".into(),
                function: OpenAiFunction {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    parameters: t.input_schema.clone(),
                },
            })
            .collect();

        let request_body = OpenAiRequest {
            model: model.into(),
            messages: openai_messages,
            stream: true,
            max_tokens: Some(16384),
            tools: openai_tools,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let response = self.client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("{} API error {}: {}", self.provider_name, status, body);
        }

        // SSE 파싱
        use tokio_stream::StreamExt;
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut usage = LlmUsage::default();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            let chunk_bytes: &[u8] = &chunk;
            buffer.push_str(&String::from_utf8_lossy(chunk_bytes));

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

                    #[derive(Deserialize)]
                    struct OpenAiChunk {
                        choices: Vec<OpenAiChoice>,
                        usage: Option<OpenAiUsage>,
                    }
                    #[derive(Deserialize)]
                    struct OpenAiChoice {
                        delta: OpenAiDelta,
                    }
                    #[derive(Deserialize)]
                    struct OpenAiDelta {
                        content: Option<String>,
                        reasoning: Option<String>,
                    }
                    #[derive(Deserialize)]
                    struct OpenAiUsage {
                        prompt_tokens: u32,
                        completion_tokens: u32,
                    }

                    if let Ok(chunk) = serde_json::from_str::<OpenAiChunk>(rest) {
                        if let Some(choice) = chunk.choices.first() {
                            if let Some(text) = &choice.delta.content {
                                let _ = tx.send(StreamChunk::TextDelta { text: text.clone() }).await;
                            }
                            if let Some(text) = &choice.delta.reasoning {
                                let _ = tx.send(StreamChunk::ThinkingDelta { text: text.clone() }).await;
                            }
                        }
                        if let Some(u) = chunk.usage {
                            usage.input_tokens = u.prompt_tokens;
                            usage.output_tokens = u.completion_tokens;
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
        let self_clone = OpenAiCompatibleProvider {
            client: self.client.clone(),
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            provider_name: self.provider_name.clone(),
        };
        let model = model.to_string();
        let messages = messages.to_vec();
        let tools = tools.to_vec();
        let thinking = thinking.clone();
        let system = system_prompt.map(|s| s.to_string());

        let handle = tokio::spawn(async move {
            self_clone.stream_chat(&model, &messages, &tools, &thinking, system.as_deref(), tx).await
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