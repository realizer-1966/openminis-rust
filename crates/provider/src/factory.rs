// 프로바이더 팩토리 — 설정 기반으로 프로바이더 인스턴스 생성
// 원본: ProviderFactory.kt

use anyhow::Result;
use crate::types::*;
use crate::traits::*;
use crate::{anthropic, openai, gemini};

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(config: &ProviderConfig) -> Result<Box<dyn LlmProvider>> {
        let provider = match config.provider_type.as_str() {
            "anthropic" => {
                let key = config.api_key.as_deref()
                    .ok_or_else(|| anyhow::anyhow!("Missing API key for anthropic"))?;
                let mut p = anthropic::AnthropicProvider::new(key.into());
                if let Some(url) = &config.base_url {
                    p = p.with_base_url(url.clone());
                }
                Box::new(p) as Box<dyn LlmProvider>
            }
            "openai" => {
                let key = config.api_key.as_deref()
                    .ok_or_else(|| anyhow::anyhow!("Missing API key for openai"))?;
                Box::new(openai::OpenAiCompatibleProvider::openai(key.into()))
                    as Box<dyn LlmProvider>
            }
            "openrouter" => {
                let key = config.api_key.as_deref()
                    .ok_or_else(|| anyhow::anyhow!("Missing API key for openrouter"))?;
                Box::new(openai::OpenAiCompatibleProvider::openrouter(key.into()))
                    as Box<dyn LlmProvider>
            }
            "xai" => {
                let key = config.api_key.as_deref()
                    .ok_or_else(|| anyhow::anyhow!("Missing API key for xai"))?;
                Box::new(openai::OpenAiCompatibleProvider::xai(key.into()))
                    as Box<dyn LlmProvider>
            }
            "gemini" => {
                let key = config.api_key.as_deref()
                    .ok_or_else(|| anyhow::anyhow!("Missing API key for gemini"))?;
                Box::new(gemini::GeminiProvider::new(key.into()))
                    as Box<dyn LlmProvider>
            }
            other => anyhow::bail!("Unknown provider type: {}", other),
        };
        Ok(provider)
    }
}