// 음성 교정 엔진 — 전체 파이프라인 (retrieve → fuse → correct)
// 원본: VoiceCorrectionEngine.kt, CorrectionStrategy.kt
//
// LLM 기반 교정: 음성 인식 결과를 LLM에 전달하여
// 문맥과 학습된 혼동 쌍으로 교정 제안을 받는다.
// 절대 크래시하지 않음 — 모든 실패 경로는 원본 텍스트 반환.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{info, warn};

use crate::correction::diff::{VoiceCorrectionDiff, CorrectionPair};
use crate::correction::levenshtein;

/// 교정 제안 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionSuggestion {
    pub original: String,
    pub corrected: String,
    pub has_change: bool,
    pub model_group_used: String,
    pub duration_ms: u64,
    pub rejected_reason: Option<String>,
    pub diff_summary: String,
    pub applied_pairs: Vec<CorrectionPair>,
}

/// 교정 후보 — 학습된 혼동 쌍
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionCandidate {
    pub from: String,
    pub to: String,
    pub confidence: f64,
    pub occurrences: u32,
}

/// 대화 컨텍스트 — 교정 프롬프트에 포함되는 최근 메시지
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversationContext {
    pub messages: Vec<String>,
}

impl ConversationContext {
    pub fn empty() -> Self {
        Self { messages: vec![] }
    }
}

/// 교정 엔진 설정
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub locale: String,
    pub max_char_change_ratio: f64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            locale: "en".into(),
            max_char_change_ratio: crate::correction::config::VoiceCorrectionConfig::MAX_CHAR_CHANGE_RATIO,
        }
    }
}

/// 음성 교정 엔진
pub struct VoiceCorrectionEngine {
    config: EngineConfig,
}

impl VoiceCorrectionEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self { config }
    }

    /// 교정 전체 파이프라인 실행
    /// 실제 LLM 호출은 외부에서 주입된 correct_fn으로 수행
    pub async fn correct(
        &self,
        transcript: &str,
        candidates: &[CorrectionCandidate],
        context: &ConversationContext,
        correct_fn: impl Fn(&str, &[CorrectionCandidate], &ConversationContext) -> Result<String>,
    ) -> CorrectionSuggestion {
        let started = Instant::now();
        let text = transcript.trim();

        if text.is_empty() {
            return CorrectionSuggestion {
                original: transcript.into(),
                corrected: transcript.into(),
                has_change: false,
                model_group_used: "none".into(),
                duration_ms: 0,
                rejected_reason: Some("empty_input".into()),
                diff_summary: String::new(),
                applied_pairs: vec![],
            };
        }

        // LLM 교정 호출
        let corrected_text = match correct_fn(text, candidates, context) {
            Ok(result) => result,
            Err(e) => {
                warn!("Correction failed: {}", e);
                return CorrectionSuggestion {
                    original: transcript.into(),
                    corrected: transcript.into(),
                    has_change: false,
                    model_group_used: "none".into(),
                    duration_ms: started.elapsed().as_millis() as u64,
                    rejected_reason: Some(format!("error: {}", e)),
                    diff_summary: String::new(),
                    applied_pairs: vec![],
                };
            }
        };

        let corrected_text = corrected_text.trim();

        // 변경 비율 검사 — 너무 많이 바뀌면 재작성으로 판단
        let ratio = VoiceCorrectionDiff::char_change_ratio(text, corrected_text);
        if ratio > self.config.max_char_change_ratio {
            info!("Correction rejected: char change ratio {:.2} > {:.2}", ratio, self.config.max_char_change_ratio);
            return CorrectionSuggestion {
                original: transcript.into(),
                corrected: transcript.into(),
                has_change: false,
                model_group_used: "llm".into(),
                duration_ms: started.elapsed().as_millis() as u64,
                rejected_reason: Some("too_many_changes".into()),
                diff_summary: format!("change_ratio: {:.2}", ratio),
                applied_pairs: vec![],
            };
        }

        // 교정 쌍 추출 (간단한 공백 토크나이저)
        let pairs = VoiceCorrectionDiff::token_pairs(text, corrected_text, |s| {
            s.split_whitespace().map(|w| w.to_string()).collect()
        });

        let has_change = text != corrected_text;
        let diff_summary = if has_change {
            format!("{} pair(s), ratio {:.2}", pairs.len(), ratio)
        } else {
            String::new()
        };

        CorrectionSuggestion {
            original: transcript.into(),
            corrected: corrected_text.into(),
            has_change,
            model_group_used: "llm".into(),
            duration_ms: started.elapsed().as_millis() as u64,
            rejected_reason: if has_change { None } else { Some("no_change".into()) },
            diff_summary,
            applied_pairs: pairs,
        }
    }
}

impl Default for VoiceCorrectionEngine {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noop_correct(_text: &str, _candidates: &[CorrectionCandidate], _ctx: &ConversationContext) -> Result<String> {
        Ok("hello".into())
    }

    #[tokio::test]
    async fn test_empty_input() {
        let engine = VoiceCorrectionEngine::default();
        let result = engine.correct("", &[], &ConversationContext::empty(), noop_correct).await;
        assert!(!result.has_change);
        assert_eq!(result.rejected_reason.unwrap(), "empty_input");
    }

    #[tokio::test]
    async fn test_correction_applied() {
        let engine = VoiceCorrectionEngine::default();
        let result = engine.correct("helllo", &[], &ConversationContext::empty(), |_, _, _| Ok("hello".into())).await;
        assert!(result.has_change);
        assert_eq!(result.corrected, "hello");
        assert_eq!(result.original, "helllo");
        assert!(result.duration_ms < 1000);
    }

    #[tokio::test]
    async fn test_too_many_changes_rejected() {
        let engine = VoiceCorrectionEngine::default();
        let result = engine.correct("hello world", &[], &ConversationContext::empty(), |_, _, _| Ok("goodbye universe".into())).await;
        assert!(!result.has_change);
        assert!(result.rejected_reason.unwrap().contains("too_many"));
    }

    #[tokio::test]
    async fn test_error_returns_original() {
        let engine = VoiceCorrectionEngine::default();
        let result = engine.correct("hello", &[], &ConversationContext::empty(), |_, _, _| Err(anyhow::anyhow!("network error"))).await;
        assert!(!result.has_change);
        assert_eq!(result.corrected, "hello");
        assert!(result.rejected_reason.unwrap().contains("error"));
    }
}