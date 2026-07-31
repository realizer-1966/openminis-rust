// 음성 교정 엔진 — 음성 인식 결과를 LLM으로 후처리 교정
// 원본: speech/correction/ (14개 Kotlin 파일)
//
// 핵심 알고리즘:
// 1. PhoneticNormalizer: 음소 정규화 (한자→병음, 유사음 통합)
// 2. Levenshtein: 편집 거리 계산 (음성 유사도 판별)
// 3. VoiceCorrectionDiff: before/after 텍스트의 문자 단위 LCS diff
// 4. VocabularyFilter: 어휘 필터 (불용어, 빈도, 품사 기반)
// 5. VoiceCorrectionEngine: 전체 파이프라인 (retrieve → fuse → correct)

pub mod phonetic;
pub mod levenshtein;
pub mod diff;
pub mod vocabulary;
pub mod config;
pub mod engine;

pub use phonetic::PhoneticNormalizer;
pub use levenshtein::levenshtein;
pub use diff::{VoiceCorrectionDiff, LcsDiff};
pub use vocabulary::{VocabularyFilter, StopWords};
pub use config::VoiceCorrectionConfig;
pub use engine::{VoiceCorrectionEngine, CorrectionSuggestion};