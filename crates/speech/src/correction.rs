// 음성 교정 엔진 — 인식 결과를 LLM으로 후처리 교정
// 원본: speech/correction/ (14개 파일)

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionResult {
    pub original: String,
    pub corrected: String,
    pub confidence: f32,
    pub diff_segments: Vec<DiffSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSegment {
    pub original: String,
    pub corrected: String,
    pub position: usize,
}

pub struct VoiceCorrectionEngine {
    vocabulary: Vec<String>,
}

impl VoiceCorrectionEngine {
    pub fn new() -> Self {
        Self { vocabulary: Vec::new() }
    }

    pub fn add_vocabulary(&mut self, words: Vec<String>) {
        self.vocabulary.extend(words);
    }

    /// 음소 정규화 — 동음이의어/유사음 교정
    pub fn normalize_phonetics(&self, text: &str) -> String {
        // TODO: PhoneticNormalizer.kt 포팅
        text.to_string()
    }

    /// 교정 실행
    pub async fn correct(&self, text: &str) -> CorrectionResult {
        // TODO: LLM 기반 교정
        CorrectionResult {
            original: text.to_string(),
            corrected: text.to_string(),
            confidence: 1.0,
            diff_segments: vec![],
        }
    }
}

impl Default for VoiceCorrectionEngine {
    fn default() -> Self { Self::new() }
}
