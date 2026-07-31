// 에이전트 루프 — LLM 응답을 받아 툴을 실행하고 결과를 다시 LLM에 전달
// 원본: AIChatViewModel+*.swift (27개 파일), ChatScreen.kt 의 에이전트 루프 부분

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};

use minis_provider::{LlmProvider, LlmMessage, LlmToolDefinition, StreamChunk, LlmUsage, ThinkingLevel, ContentPart, Role};
use crate::tool_dispatch::{ToolDispatcher, ToolResult};

/// 에이전트 루프 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLoopConfig {
    /// 최대 툴 호출 반복 횟수 (무한 루프 방지)
    pub max_iterations: u32,
    /// 동시 툴 실행 허용 여부
    pub concurrent_tools: bool,
    /// 컨텍스트 압축 임계값 (토큰 수)
    pub compaction_threshold: usize,
}

impl Default for AgentLoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 50,
            concurrent_tools: true,
            compaction_threshold: 160_000,
        }
    }
}

/// 에이전트 루프 이벤트 — UI/Tauri에 전달
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentEvent {
    /// LLM 스트리밍 토큰
    Token { text: String },
    /// 툴 실행 시작
    ToolStarted { tool_name: String, tool_id: String },
    /// 툴 실행 완료
    ToolFinished { tool_id: String, success: bool, output: String },
    /// thinking 블록
    Thinking { text: String },
    /// 토큰 사용량 업데이트
    Usage { input_tokens: u32, output_tokens: u32 },
    /// 에이전트 완료
    Done,
    /// 에러
    Error { message: String },
}

/// 툴 콜 — LLM이 요청한 툴 실행
#[derive(Debug, Clone)]
struct ToolCall {
    id: String,
    name: String,
    input: serde_json::Value,
}

/// 에이전트 루프 — LLM과 툴을 번갈아 실행
pub struct AgentLoop {
    config: AgentLoopConfig,
    dispatcher: Arc<ToolDispatcher>,
}

impl AgentLoop {
    pub fn new(config: AgentLoopConfig, dispatcher: Arc<ToolDispatcher>) -> Self {
        Self { config, dispatcher }
    }

    /// 에이전트 실행 — 사용자 메시지로 시작
    /// event_tx로 스트리밍 이벤트를 전송하며, 완료 시 최종 응답 텍스트 반환
    pub async fn run(
        &self,
        provider: &dyn LlmProvider,
        model: &str,
        system_prompt: &str,
        user_message: &str,
        tools: &[LlmToolDefinition],
        thinking: &ThinkingLevel,
        event_tx: mpsc::Sender<AgentEvent>,
    ) -> Result<String> {
        let mut messages: Vec<LlmMessage> = vec![
            LlmMessage::user(user_message),
        ];

        let mut final_text = String::new();

        for iteration in 0..self.config.max_iterations {
            info!("Agent loop iteration {}", iteration + 1);

            // LLM 스트리밍 호출 — 동기적으로 스트림 처리
            let (stream_tx, mut rx) = mpsc::channel::<StreamChunk>(256);

            let stream_result = provider.stream_chat(
                model,
                &messages,
                tools,
                thinking,
                Some(system_prompt),
                stream_tx,
            ).await;

            // 스트림 처리 — 토큰/thinking/tool_use 수집
            let mut iteration_text = String::new();
            let mut iteration_thinking = String::new();
            let mut tool_calls: Vec<ToolCall> = Vec::new();
            let mut current_tool_id = String::new();
            let mut current_tool_name = String::new();
            let mut current_tool_json = String::new();
            let mut usage = LlmUsage::default();

            while let Some(chunk) = rx.recv().await {
                match chunk {
                    StreamChunk::TextDelta { text } => {
                        iteration_text.push_str(&text);
                        let _ = event_tx.send(AgentEvent::Token { text }).await;
                    }
                    StreamChunk::ThinkingDelta { text } => {
                        iteration_thinking.push_str(&text);
                        let _ = event_tx.send(AgentEvent::Thinking { text }).await;
                    }
                    StreamChunk::ToolUseStart { id, name } => {
                        current_tool_id = id.clone();
                        current_tool_name = name.clone();
                        current_tool_json.clear();
                        let _ = event_tx.send(AgentEvent::ToolStarted {
                            tool_name: name,
                            tool_id: id,
                        }).await;
                    }
                    StreamChunk::ToolUseDelta { partial_json, .. } => {
                        current_tool_json.push_str(&partial_json);
                    }
                    StreamChunk::ToolUseStop { id: _ } => {
                        // 툴 콜 완료 — 파싱해서 큐에 추가
                        let input = if current_tool_json.is_empty() {
                            serde_json::Value::Null
                        } else {
                            serde_json::from_str(&current_tool_json)
                                .unwrap_or(serde_json::Value::Null)
                        };
                        tool_calls.push(ToolCall {
                            id: current_tool_id.clone(),
                            name: current_tool_name.clone(),
                            input,
                        });
                        debug!("Tool call queued: {} (id={})", current_tool_name, current_tool_id);
                        current_tool_id.clear();
                        current_tool_name.clear();
                        current_tool_json.clear();
                    }
                    StreamChunk::Usage { input_tokens, output_tokens } => {
                        usage.input_tokens = input_tokens;
                        usage.output_tokens = output_tokens;
                        let _ = event_tx.send(AgentEvent::Usage {
                            input_tokens,
                            output_tokens,
                        }).await;
                    }
                    StreamChunk::Done => break,
                    StreamChunk::Error { message } => {
                        let _ = event_tx.send(AgentEvent::Error { message }).await;
                        anyhow::bail!("LLM stream error");
                    }
                }
            }

            // 스트림 완료 — usage 처리
            let final_usage = stream_result?;
            if usage.input_tokens == 0 {
                usage = final_usage;
            }

            if !iteration_text.is_empty() {
                final_text.push_str(&iteration_text);
            }

            // 툴 콜이 없으면 에이전트 완료
            if tool_calls.is_empty() {
                info!("Agent loop complete — no tool calls, {} chars", iteration_text.len());
                let _ = event_tx.send(AgentEvent::Done).await;
                break;
            }

            // assistant 메시지 추가 (텍스트 + 툴 콜)
            let mut assistant_content = Vec::new();
            if !iteration_text.is_empty() {
                assistant_content.push(ContentPart::Text { text: iteration_text });
            }
            for tc in &tool_calls {
                assistant_content.push(ContentPart::ToolUse {
                    id: tc.id.clone(),
                    name: tc.name.clone(),
                    input: tc.input.clone(),
                });
            }
            messages.push(LlmMessage {
                role: Role::Assistant,
                content: assistant_content,
            });

            // 툴 실행
            for tc in &tool_calls {
                info!("Executing tool: {} (id={})", tc.name, tc.id);

                let result = match self.dispatcher.execute(&tc.name, tc.input.clone()).await {
                    Ok(result) => result,
                    Err(e) => {
                        warn!("Tool {} failed: {}", tc.name, e);
                        ToolResult {
                            tool_name: tc.name.clone(),
                            tool_id: tc.id.clone(),
                            success: false,
                            output: format!("Error: {}", e),
                            minis_urls: vec![],
                        }
                    }
                };

                let _ = event_tx.send(AgentEvent::ToolFinished {
                    tool_id: tc.id.clone(),
                    success: result.success,
                    output: result.output.clone(),
                }).await;

                // 툴 결과를 user 메시지로 추가 (Anthropic 형식)
                messages.push(LlmMessage {
                    role: Role::User,
                    content: vec![ContentPart::ToolResult {
                        tool_use_id: tc.id.clone(),
                        content: result.output,
                        is_error: !result.success,
                    }],
                });
            }

            // 컨텍스트 압축 체크 (간단한 휴리스틱)
            let total_chars: usize = messages.iter()
                .map(|m| m.content.iter()
                    .filter_map(|p| match p {
                        ContentPart::Text { text } => Some(text.len()),
                        _ => None,
                    })
                    .sum::<usize>())
                .sum();
            if total_chars > self.config.compaction_threshold {
                warn!("Context compaction needed ({} chars > {} threshold)", 
                      total_chars, self.config.compaction_threshold);
                // TODO: 실제 압축 구현 (이전 메시지 요약)
            }
        }

        // 최대 반복 도달
        if !final_text.is_empty() {
            let _ = event_tx.send(AgentEvent::Done).await;
        } else {
            let _ = event_tx.send(AgentEvent::Error {
                message: format!("Max iterations ({}) reached without completion", self.config.max_iterations),
            }).await;
        }

        Ok(final_text)
    }

    pub fn config(&self) -> &AgentLoopConfig {
        &self.config
    }

    pub fn dispatcher(&self) -> &ToolDispatcher {
        &self.dispatcher
    }
}