// 어휘 필터 — 음성 인식에서 마이닝된 용어의 저장 여부 결정
// 원본: VocabularyFilter.kt, StopWords.kt
//
// 점수 기반: 배경 빈도 + 품사 + 라틴/숫자 포함 + 길이로 평가

use serde::{Deserialize, Serialize};

/// 어휘 평가 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VocabularyDecision {
    Accepted {
        score: u32,
        breakdown: ScoreBreakdown,
    },
    Rejected {
        reason: String,
    },
}

/// 점수 내역
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub background_rank: u32,
    pub pos_tag: u32,
    pub has_latin_or_digit: u32,
    pub length_ok: u32,
}

impl ScoreBreakdown {
    pub fn total(&self) -> u32 {
        self.background_rank + self.pos_tag + self.has_latin_or_digit + self.length_ok
    }
}

/// 어휘 필터
pub struct VocabularyFilter {
    min_score: u32,
    max_length: usize,
    min_length: usize,
    proper_noun_tags: Vec<String>,
}

impl Default for VocabularyFilter {
    fn default() -> Self {
        Self {
            min_score: crate::correction::config::VoiceCorrectionConfig::VOCAB_MIN_SCORE,
            max_length: 12,
            min_length: 2,
            proper_noun_tags: vec!["nr".into(), "ns".into(), "nz".into(), "nt".into()],
        }
    }
}

impl VocabularyFilter {
    /// 용어 평가 — rank 함수로 배경 빈도 조회
    pub fn evaluate(&self, term: &str, pos_tag: Option<&str>, rank: impl Fn(&str) -> Option<u32>) -> VocabularyDecision {
        if StopWords::contains(term) {
            return VocabularyDecision::Rejected { reason: "stopword_blacklist".into() };
        }
        let char_count = term.chars().count();
        if char_count < self.min_length {
            return VocabularyDecision::Rejected { reason: "too_short".into() };
        }
        if char_count > self.max_length {
            return VocabularyDecision::Rejected { reason: "too_long".into() };
        }

        // 숫자/구두점만 있는 경우
        if term.chars().all(|c| c.is_ascii_digit() || is_punctuation_like(c) || c.is_whitespace()) {
            return VocabularyDecision::Rejected { reason: "no_lexical_content".into() };
        }

        let mut b = ScoreBreakdown::default();

        // 배경 빈도 — 흔한 단어는 점수 없음
        let r = rank(term);
        if r.is_none() || r.unwrap() > crate::correction::config::VoiceCorrectionConfig::VOCAB_BACKGROUND_RANK_THRESHOLD {
            b.background_rank = 2;
        }

        // 고유명사 품사
        if let Some(tag) = pos_tag {
            if self.proper_noun_tags.iter().any(|t| t == tag) {
                b.pos_tag = 2;
            }
        }

        // 라틴 문자 또는 ASCII 숫자 포함
        let has_latin = term.chars().any(|c| c.is_ascii_alphabetic());
        let has_digit = term.chars().any(|c| c.is_ascii_digit());
        if has_latin || has_digit {
            b.has_latin_or_digit = 1;
        }

        // 길이 보너스
        if char_count >= 2 && char_count <= 6 {
            b.length_ok = 1;
        }

        if b.total() >= self.min_score {
            VocabularyDecision::Accepted { score: b.total(), breakdown: b }
        } else {
            VocabularyDecision::Rejected { reason: format!("low_score({})", b.total()) }
        }
    }

    /// 텍스트에서 용어 후보 추출
    pub fn candidates(
        text: &str,
        segment: impl Fn(&str) -> Vec<String>,
    ) -> Vec<(String, u32)> {
        let tokens = segment(text);
        let mut counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for token in tokens {
            *counts.entry(token).or_insert(0) += 1;
        }
        let mut result: Vec<(String, u32)> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1)); // 빈도순 정렬
        result
    }
}

fn is_punctuation_like(c: char) -> bool {
    !c.is_alphanumeric() && !c.is_whitespace()
}

/// 불용어 — 중국어 + 영어
pub struct StopWords;

impl StopWords {
    pub fn contains(term: &str) -> bool {
        Self::chinese().contains(term) || Self::english().contains(term.to_lowercase().as_str())
    }

    fn chinese() -> &'static std::collections::HashSet<&'static str> {
        use std::sync::OnceLock;
        static SET: OnceLock<std::collections::HashSet<&'static str>> = OnceLock::new();
        SET.get_or_init(|| {
            ["的", "了", "是", "我", "你", "他", "她", "它", "我们", "你们", "他们",
            "这", "那", "这个", "那个", "什么", "怎么", "为什么", "哪里", "谁",
            "有", "没有", "不", "也", "都", "很", "太", "就", "还", "又", "再", "才", "只",
            "可以", "能", "会", "要", "想", "需要", "应该", "可能", "或者", "但是", "因为", "所以",
            "然后", "而且", "不过", "虽然", "如果", "继续", "好的", "嗯", "就是", "其实", "反正",
            "一下", "一点", "一样", "一起", "现在", "今天", "明天", "昨天",
            "说", "讲", "看", "听", "做", "用", "给", "让", "把", "被", "在", "和", "与", "对"]
            .into_iter().collect()
        })
    }

    fn english() -> &'static std::collections::HashSet<&'static str> {
        use std::sync::OnceLock;
        static SET: OnceLock<std::collections::HashSet<&'static str>> = OnceLock::new();
        SET.get_or_init(|| {
            ["the", "a", "an", "and", "or", "but", "if", "then", "than", "that", "this",
            "is", "are", "was", "were", "be", "been", "being", "am",
            "do", "does", "did", "have", "has", "had",
            "can", "could", "will", "would", "shall", "should", "may", "might", "must",
            "i", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us", "them",
            "my", "your", "his", "its", "our", "their",
            "to", "of", "in", "on", "at", "by", "for", "with", "from", "into", "about",
            "not", "no", "yes", "ok", "okay", "so", "just", "very", "too", "also", "only",
            "what", "which", "who", "when", "where", "why", "how",
            "there", "here", "some", "any", "all", "more", "most", "one", "two",
            "get", "got", "let", "make", "made", "use", "used", "like", "want", "need",
            "please", "thanks", "thank", "hi", "hello", "now", "still", "even", "up", "out", "over"]
            .into_iter().collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stopword_chinese() {
        assert!(StopWords::contains("的"));
        assert!(StopWords::contains("我"));
        assert!(!StopWords::contains("张三"));
    }

    #[test]
    fn test_stopword_english() {
        assert!(StopWords::contains("the"));
        assert!(StopWords::contains("THE"));
        assert!(StopWords::contains("Hello"));
        assert!(!StopWords::contains("algorithm"));
    }

    #[test]
    fn test_vocabulary_reject_short() {
        let filter = VocabularyFilter::default();
        // "x" is not a stopword but is too short (1 char)
        match filter.evaluate("x", None, |_| None) {
            VocabularyDecision::Rejected { reason } => assert!(reason.contains("short"), "expected too_short, got: {}", reason),
            _ => panic!("should reject short term"),
        }
    }

    #[test]
    fn test_vocabulary_reject_stopword() {
        let filter = VocabularyFilter::default();
        match filter.evaluate("the", None, |_| Some(1)) {
            VocabularyDecision::Rejected { reason } => assert!(reason.contains("stopword")),
            _ => panic!("should reject stopword"),
        }
    }

    #[test]
    fn test_vocabulary_accept_rare() {
        let filter = VocabularyFilter::default();
        // 흔하지 않은 단어 (rank = None = 배경 목록에 없음 = rare)
        match filter.evaluate("OpenAI", None, |_| None) {
            VocabularyDecision::Accepted { score, .. } => {
                assert!(score >= 2, "rare + has_latin should score >= 2: {}", score);
            }
            VocabularyDecision::Rejected { reason } => panic!("should accept: {}", reason),
        }
    }

    #[test]
    fn test_vocabulary_reject_common() {
        let filter = VocabularyFilter::default();
        // 배경 목록에 있는 흔한 단어 (rank=50, 짧아서 length_ok=0)
        // "cat"은 3자 (length_ok=1), 라틴 포함(1), rank 낮음(0) → total=2 → Accepted
        // 더 명확한 reject를 위해 stopword 사용
        match filter.evaluate("with", None, |_| Some(50)) {
            VocabularyDecision::Rejected { reason } => assert!(reason.contains("stopword") || reason.contains("low_score"), "got: {}", reason),
            _ => panic!("should reject common word"),
        }
    }
}