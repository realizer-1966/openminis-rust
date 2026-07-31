// 컨텍스트 압축 — 대화가 길어지면 이전 메시지를 요약
// 원본: AIChatViewModel+Compaction.swift

use minis_provider::{LlmMessage, ContentPart, Role, ThinkingLevel};
use anyhow::Result;
use tracing::info;

/// 압축 설정
#[derive(Debug, Clone)]
pub struct CompactionConfig {
    /// 압축 임계값 (문자 수 기준, 대략 토큰 수 * 4)
    pub char_threshold: usize,
    /// 압축 후 유지할 최근 메시지 수
    pub keep_recent: usize,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            char_threshold: 160_000 * 4, // compaction_threshold * 4
            keep_recent: 6,
        }
    }
}

/// 메시지 목록의 총 문자 수 계산
pub fn total_chars(messages: &[LlmMessage]) -> usize {
    messages.iter()
        .map(|m| m.content.iter()
            .filter_map(|p| match p {
                ContentPart::Text { text } => Some(text.len()),
                _ => None,
            })
            .sum::<usize>())
        .sum()
}

/// 압축이 필요한지 확인
pub fn needs_compaction(messages: &[LlmMessage], config: &CompactionConfig) -> bool {
    total_chars(messages) > config.char_threshold
}

/// 이전 메시지를 요약으로 교체
/// 최근 keep_recent개 메시지는 유지, 나머지는 요약 텍스트로 대체
pub fn compact_messages(
    messages: &[LlmMessage],
    config: &CompactionConfig,
) -> Vec<LlmMessage> {
    if messages.len() <= config.keep_recent {
        return messages.to_vec();
    }

    let split_point = messages.len() - config.keep_recent;
    let old_messages = &messages[..split_point];
    let recent_messages = &messages[split_point..];

    let summary_text_owned: String = old_messages.iter()
        .filter_map(|m| {
            let text: String = m.content.iter()
                .filter_map(|p| match p {
                    ContentPart::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            if text.is_empty() {
                None
            } else {
                Some(format!("[{}] {}",
                    match m.role {
                        Role::User => "User",
                        Role::Assistant => "Assistant",
                        Role::System => "System",
                        Role::Tool => "Tool",
                    },
                    text))
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let old_char_count = total_chars(old_messages);
    let summary_len = summary_text_owned.len();

    // 요약 메시지 (시스템 역할)
    let summary_body = if summary_text_owned.len() > 8000 {
        let truncated = summary_text_owned[..8000].to_string();
        format!("{}...(truncated, {} total chars)", truncated, summary_len)
    } else {
        summary_text_owned
    };

    let summary = LlmMessage::system(&format!(
        "Previous conversation summary (compacted):\n{}\n\n[End of summary. Recent messages follow.]",
        summary_body
    ));

    info!("Compacted {} messages into summary ({} chars → {} chars)",
          old_messages.len(), old_char_count, summary_len);

    let mut result = vec![summary];
    result.extend(recent_messages.iter().cloned());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_chars() {
        let msgs = vec![
            LlmMessage::user("Hello world"),
            LlmMessage::assistant("Hi there!"),
        ];
        assert_eq!(total_chars(&msgs), 20); // "Hello world" (11) + "Hi there!" (9)
    }

    #[test]
    fn test_compact_short() {
        let msgs = vec![LlmMessage::user("Hi")];
        let config = CompactionConfig { char_threshold: 100, keep_recent: 6 };
        let result = compact_messages(&msgs, &config);
        assert_eq!(result.len(), 1); // 압축 안 함
    }

    #[test]
    fn test_compact_long() {
        let mut msgs = Vec::new();
        for i in 0..20 {
            msgs.push(LlmMessage::user(&format!("Message number {} with some text content here", i)));
            msgs.push(LlmMessage::assistant(&format!("Response to message {}", i)));
        }
        let config = CompactionConfig { char_threshold: 100, keep_recent: 4 };
        let result = compact_messages(&msgs, &config);
        assert!(result.len() < msgs.len());
        assert!(result.len() > 0);
    }
}