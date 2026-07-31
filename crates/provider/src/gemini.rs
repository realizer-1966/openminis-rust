// Gemini (Google) 프로바이더
// 원본: provider/gemini/ 디렉토리
// Gemini는 OpenAI 호환 엔드포인트도 제공하지만, 네이티브 API가 더 풍부한 기능 지원

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;

use crate::types::*;
use crate::traits::LlmProvider;

pub struct GeminiProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://generativelanguage.googleapis.com".into(),
        }
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    fn provider_type(&self) -> &str { "gemini" }

    async fn list_models(&self) -> Result<Vec<ModelEntry>> {
        let url = format!("{}/v1beta/models?key={}", self.base_url, self.api_key);
        let response = self.client.get(&url).send().await?;

        #[derive(Deserialize)]
        struct GeminiModelsResponse {
            models: Vec<GeminiModel>,
        }
        #[derive(Deserialize)]
        struct GeminiModel {
            name: String,
            displayName: String,
            #[serde(default)]
            inputTokenLimit: Option<u32>,
            #[serde(default)]
            outputTokenLimit: Option<u32>,
        }

        let resp: GeminiModelsResponse = response.json().await?;
        Ok(resp.models.into_iter().map(|m| ModelEntry {
            id: m.name.replace("models/", ""),
            name: m.displayName.clone(),
            provider: "gemini".into(),
            context_window: m.inputTokenLimit,
            max_output_tokens: m.outputTokenLimit,
            supports_tools: true,
            supports_vision: true,
            supports_thinking: false,
        }).collect())
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
        // Gemini는 OpenAI 호환 엔드포인트를 사용해 스트리밍 지원
        // https://generativelanguage.googleapis.com/v1beta/openai/chat/completions
        let openai_compat = crate::openai::OpenAiCompatibleProvider::new(
            "gemini",
            self.api_key.clone(),
            format!("{}/v1beta/openai", self.base_url),
        );
        openai_compat.stream_chat(model, messages, tools, thinking, system_prompt, tx).await
    }

    async fn chat(
        &self,
        model: &str,
        messages: &[LlmMessage],
        tools: &[LlmToolDefinition],
        thinking: &ThinkingLevel,
        system_prompt: Option<&str>,
    ) -> Result<(String, LlmUsage)> {
        let openai_compat = crate::openai::OpenAiCompatibleProvider::new(
            "gemini",
            self.api_key.clone(),
            format!("{}/v1beta/openai", self.base_url),
        );
        openai_compat.chat(model, messages, tools, thinking, system_prompt).await
    }
}