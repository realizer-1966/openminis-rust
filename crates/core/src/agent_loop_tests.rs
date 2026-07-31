// 에이전트 루프 통합 테스트
// Mock 프로바이더로 LLM ↔ 툴 루프가 정상 작동하는지 검증

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_provider::MockProvider;
    use crate::tool_dispatch::{ToolDispatcher, Tool, ToolDefinition, ToolResult};
    use crate::agent_loop::{AgentLoop, AgentLoopConfig, AgentEvent};
    use minis_provider::{LlmProvider, LlmMessage, LlmToolDefinition, StreamChunk, LlmUsage, ThinkingLevel, ModelEntry, ContentPart, Role};
    use async_trait::async_trait;
    use serde_json::{json, Value};
    use std::sync::Arc;

    // 테스트용 echo 툴
    struct EchoTool;
    #[async_trait]
    impl Tool for EchoTool {
        fn name(&self) -> &str { "echo" }
        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: "echo".into(),
                description: "Echo back the input text".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "text": { "type": "string" }
                    },
                    "required": ["text"]
                }),
            }
        }
        async fn execute(&self, params: Value) -> anyhow::Result<ToolResult> {
            let text = params["text"].as_str().unwrap_or("no text");
            Ok(ToolResult {
                tool_name: "echo".into(),
                tool_id: String::new(),
                success: true,
                output: format!("Echo: {}", text),
                minis_urls: vec![],
            })
        }
    }

    #[tokio::test]
    async fn test_agent_loop_no_tools() {
        // 툴 콜 없이 바로 응답
        let provider = MockProvider::simple("Hello! How can I help you?");
        let dispatcher = Arc::new(ToolDispatcher::new());
        let loop_config = AgentLoopConfig::default();
        let agent = AgentLoop::new(loop_config, dispatcher);

        let (tx, mut rx) = tokio::sync::mpsc::channel(256);
        let tools: Vec<minis_provider::LlmToolDefinition> = vec![];

        let result = agent.run(
            &provider,
            "mock-model",
            "You are a helpful assistant.",
            "Hi there",
            &tools,
            &minis_provider::ThinkingLevel::None,
            tx,
        ).await;

        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("Hello! How can I help you?"));

        // Done 이벤트 확인
        let mut got_done = false;
        while let Ok(event) = rx.try_recv() {
            if let AgentEvent::Done = event {
                got_done = true;
            }
        }
        assert!(got_done);
    }

    #[tokio::test]
    async fn test_agent_loop_with_tool_call() {
        // 1차: 툴 콜 → 2차: 툴 결과 받고 최종 응답
        let provider = MockProvider::with_tool_call(
            "echo",
            r#"{"text":"world"}"#,
            "I echoed 'world' for you!",
        );

        let mut dispatcher = ToolDispatcher::new();
        dispatcher.register(Box::new(EchoTool));
        let dispatcher = Arc::new(dispatcher);

        let agent = AgentLoop::new(AgentLoopConfig::default(), dispatcher);
        let (tx, mut rx) = tokio::sync::mpsc::channel(256);
        let tools = agent.dispatcher().llm_definitions();

        let result = agent.run(
            &provider,
            "mock-model",
            "You are a helpful assistant.",
            "Echo the word 'world'",
            &tools,
            &minis_provider::ThinkingLevel::None,
            tx,
        ).await;

        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("I echoed"));

        // 툴 실행 이벤트 확인
        let mut tool_started = false;
        let mut tool_finished = false;
        let mut got_done = false;
        while let Ok(event) = rx.try_recv() {
            match event {
                AgentEvent::ToolStarted { tool_name, .. } => {
                    assert_eq!(tool_name, "echo");
                    tool_started = true;
                }
                AgentEvent::ToolFinished { success, .. } => {
                    assert!(success);
                    tool_finished = true;
                }
                AgentEvent::Done => got_done = true,
                _ => {}
            }
        }
        assert!(tool_started, "ToolStarted event missing");
        assert!(tool_finished, "ToolFinished event missing");
        assert!(got_done, "Done event missing");
    }

    #[tokio::test]
    async fn test_agent_loop_max_iterations() {
        // 계속 툴만 부르는 mock — max_iterations에 도달해야 함
        let tool_call_chunk = vec![
            StreamChunk::ToolUseStart { id: "tool_1".into(), name: "echo".into() },
            StreamChunk::ToolUseDelta { id: "tool_1".into(), partial_json: r#"{"text":"loop"}"#.into() },
            StreamChunk::ToolUseStop { id: "tool_1".into() },
            StreamChunk::Done,
        ];
        // 60번의 툴 콜 응답 (max_iterations=50보다 많게)
        let responses: Vec<Vec<StreamChunk>> = (0..60).map(|_| tool_call_chunk.clone()).collect();
        let provider = MockProvider::new(responses);

        let mut dispatcher = ToolDispatcher::new();
        dispatcher.register(Box::new(EchoTool));
        let dispatcher = Arc::new(dispatcher);

        let config = AgentLoopConfig {
            max_iterations: 5, // 낮게 설정
            ..Default::default()
        };
        let agent = AgentLoop::new(config, dispatcher);
        let (tx, _rx) = tokio::sync::mpsc::channel(256);
        let tools = agent.dispatcher().llm_definitions();

        let result = agent.run(
            &provider,
            "mock-model",
            "You are a helpful assistant.",
            "Keep calling echo",
            &tools,
            &minis_provider::ThinkingLevel::None,
            tx,
        ).await;

        // max_iterations 도달 — 에러 또는 빈 텍스트
        // (에이전트가 max_iterations에 도달하면 Error 이벤트 전송)
        assert!(result.is_ok()); // run 자체는 Ok — 에러는 이벤트로 전송
    }
}