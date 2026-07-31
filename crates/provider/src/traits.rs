// LLM 프로바이더 trait — 모든 프로바이더가 구현
// 원본: LLMProvider.kt (interface)

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;
use crate::types::*;

/// 프로바이더가 구현해야 하는 trait
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// 프로바이더 식별자 (e.g. "anthropic", "openai")
    fn provider_type(&self) -> &str;

    /// 사용 가능한 모델 목록
    async fn list_models(&self) -> Result<Vec<ModelEntry>>;

    /// 스트리밍 채팅 — 청크를 channel로 전송
    async fn stream_chat(
        &self,
        model: &str,
        messages: &[LlmMessage],
        tools: &[LlmToolDefinition],
        thinking: &ThinkingLevel,
        system_prompt: Option<&str>,
        tx: mpsc::Sender<StreamChunk>,
    ) -> Result<LlmUsage>;

    /// 비스트리밍 채팅 (간단한 요청용)
    async fn chat(
        &self,
        model: &str,
        messages: &[LlmMessage],
        tools: &[LlmToolDefinition],
        thinking: &ThinkingLevel,
        system_prompt: Option<&str>,
    ) -> Result<(String, LlmUsage)>;
}

/// 프로바이더 인스턴스 — 설정 기반으로 생성
pub struct ProviderInstance {
    pub config: ProviderConfig,
    pub provider: Box<dyn LlmProvider>,
}