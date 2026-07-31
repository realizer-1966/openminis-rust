// 에이전트 루프 — LLM 응답을 받아 툴을 실행하고 결과를 다시 LLM에 전달
// 원본: AIChatViewModel+*.swift (27개 파일), ChatScreen.kt 의 에이전트 루프 부분

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

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

/// 에이전트 루프 이벤트 — UI에 전달
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentEvent {
    /// LLM 스트리밍 토큰
    Token { text: String },
    /// 툴 실행 시작
    ToolStarted { tool_name: String, tool_id: String },
    /// 툴 실행 완료
    ToolFinished { tool_id: String, success: bool },
    /// thinking 블록
    Thinking { text: String },
    /// 에이전트 완료
    Done,
    /// 에러
    Error { message: String },
}

/// 에이전트 루프
pub struct AgentLoop {
    config: AgentLoopConfig,
}

impl AgentLoop {
    pub fn new(config: AgentLoopConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &AgentLoopConfig {
        &self.config
    }
}