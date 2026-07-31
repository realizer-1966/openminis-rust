// models.dev API — 모델 메타데이터 자동 업데이트
// 원본: ModelsDevApi.kt, ModelsDevAPI.swift

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use crate::types::ModelEntry;

pub struct ModelsDevApi {
    client: Client,
    base_url: String,
}

impl ModelsDevApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://models.dev".into(),
        }
    }

    pub async fn fetch_models(&self) -> Result<Vec<ModelEntry>> {
        let url = format!("{}/api/models", self.base_url);
        let response = self.client.get(&url).send().await?;

        #[derive(Deserialize)]
        struct ModelsDevResponse {
            models: Vec<ModelsDevModel>,
        }
        #[derive(Deserialize)]
        struct ModelsDevModel {
            id: String,
            name: String,
            provider: String,
            #[serde(default)]
            context_length: Option<u32>,
            #[serde(default)]
            max_output: Option<u32>,
            #[serde(default)]
            tools: bool,
            #[serde(default)]
            vision: bool,
            #[serde(default)]
            reasoning: bool,
        }

        let resp: ModelsDevResponse = response.json().await?;
        Ok(resp.models.into_iter().map(|m| {
            let id = m.id;
            let name = m.name.clone();
            let provider = m.provider.clone();
            ModelEntry {
                id,
                name,
                provider,
                context_window: m.context_length,
                max_output_tokens: m.max_output,
                supports_tools: m.tools,
                supports_vision: m.vision,
                supports_thinking: m.reasoning,
            }
        }).collect())
    }
}

impl Default for ModelsDevApi {
    fn default() -> Self { Self::new() }
}