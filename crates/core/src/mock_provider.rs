// 테스트용 Mock LLM 프로바이더 — 실제 API 호출 없이 에이전트 루프 테스트

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;
use minis_provider::{LlmProvider, LlmMessage, LlmToolDefinition, StreamChunk, LlmUsage, ThinkingLevel, ModelEntry};

/// Mock 프로바이더 — 미리 정의된 응답 시퀀스를 반환
pub struct MockProvider {
    /// 각 stream_chat 호출 시 반환할 청크 시퀀스
    responses: Vec<Vec<StreamChunk>>,
    call_count: std::sync::Mutex<usize>,
}

impl MockProvider {
    pub fn new(responses: Vec<Vec<StreamChunk>>) -> Self {
        Self {
            responses,
            call_count: std::sync::Mutex::new(0),
        }
    }

    /// 툴 콜 후 최종 텍스트 응답을 반환하는 간단한 mock
    pub fn with_tool_call(tool_name: &str, tool_input: &str, final_text: &str) -> Self {
        Self::new(vec![
            // 첫 번째 호출: 툴 콜
            vec![
                StreamChunk::TextDelta { text: "Let me check that. ".into() },
                StreamChunk::ToolUseStart { id: "tool_1".into(), name: tool_name.into() },
                StreamChunk::ToolUseDelta { id: "tool_1".into(), partial_json: tool_input.into() },
                StreamChunk::ToolUseStop { id: "tool_1".into() },
                StreamChunk::Usage { input_tokens: 100, output_tokens: 50 },
                StreamChunk::Done,
            ],
            // 두 번째 호출: 툴 결과를 받고 최종 응답
            vec![
                StreamChunk::TextDelta { text: final_text.into() },
                StreamChunk::Usage { input_tokens: 200, output_tokens: 30 },
                StreamChunk::Done,
            ],
        ])
    }

    /// 툴 콜 없이 바로 응답
    pub fn simple(text: &str) -> Self {
        Self::new(vec![
            vec![
                StreamChunk::TextDelta { text: text.into() },
                StreamChunk::Usage { input_tokens: 50, output_tokens: 20 },
                StreamChunk::Done,
            ],
        ])
    }
}

#[async_trait]
impl LlmProvider for MockProvider {
    fn provider_type(&self) -> &str { "mock" }

    async fn list_models(&self) -> Result<Vec<ModelEntry>> {
        Ok(vec![ModelEntry {
            id: "mock-model".into(),
            name: "Mock Model".into(),
            provider: "mock".into(),
            context_window: Some(100_000),
            max_output_tokens: Some(4_096),
            supports_tools: true,
            supports_vision: false,
            supports_thinking: false,
        }])
    }

    async fn stream_chat(
        &self,
        _model: &str,
        _messages: &[LlmMessage],
        _tools: &[LlmToolDefinition],
        _thinking: &ThinkingLevel,
        _system_prompt: Option<&str>,
        tx: mpsc::Sender<StreamChunk>,
    ) -> Result<LlmUsage> {
        let idx = {
            let mut count = self.call_count.lock().unwrap();
            let idx = *count;
            *count += 1;
            idx
        };

        let chunks = self.responses.get(idx)
            .ok_or_else(|| anyhow::anyhow!("MockProvider: no more responses (call {})", idx))?;

        let mut usage = LlmUsage::default();
        for chunk in chunks {
            if let StreamChunk::Usage { input_tokens, output_tokens } = chunk {
                usage.input_tokens = *input_tokens;
                usage.output_tokens = *output_tokens;
            }
            let _ = tx.send(chunk.clone()).await;
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
        let usage = self.stream_chat(model, messages, tools, thinking, system_prompt, tx).await?;
        let mut text = String::new();
        while let Some(chunk) = rx.recv().await {
            if let StreamChunk::TextDelta { text: t } = chunk {
                text.push_str(&t);
            }
        }
        Ok((text, usage))
    }
}