// LLM 공통 타입 — 메시지, 툴 콜, 스트리밍 청크, 사용량
// 원본: data/model/LLMMessage.kt, LLMStreamChunk.kt, LLMUsage.kt, AgentToolDefinition.kt

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 채팅 메시지 역할
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// 메시지 콘텐츠 파트 — 텍스트, 이미지, 툴 콜, 툴 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    Text { text: String },
    Image { source: ImageSource },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: String, is_error: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    pub media_type: String,
    pub data: String, // base64
}

/// LLM 메시지
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: Role,
    pub content: Vec<ContentPart>,
}

impl LlmMessage {
    pub fn user(text: &str) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentPart::Text { text: text.into() }],
        }
    }

    pub fn system(text: &str) -> Self {
        Self {
            role: Role::System,
            content: vec![ContentPart::Text { text: text.into() }],
        }
    }

    pub fn assistant(text: &str) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentPart::Text { text: text.into() }],
        }
    }
}

/// 툴 정의 (LLM에게 전달)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// 스트리밍 청크
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamChunk {
    TextDelta { text: String },
    ThinkingDelta { text: String },
    ToolUseStart { id: String, name: String },
    ToolUseDelta { id: String, partial_json: String },
    ToolUseStop { id: String },
    Usage { input_tokens: u32, output_tokens: u32 },
    Done,
    Error { message: String },
}

/// 토큰 사용량
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlmUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_tokens: u32,
    pub cache_creation_tokens: u32,
}

/// 모델 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_window: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
}

/// 프로바이더 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub provider_type: String, // "anthropic", "openai", "gemini", etc.
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub models: Vec<ModelEntry>,
    pub created_at: DateTime<Utc>,
}

/// thinking 레벨
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThinkingLevel {
    None,
    Low,
    Medium,
    High,
    Max,
}