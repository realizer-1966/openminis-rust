// 음성 파이프라인 — 교정 엔진, VAD, 문장 분할
// 원본: speech/ 디렉토리 (14개 correction/ 파일 + 음성 인식)

pub mod correction;
pub mod vad;
pub mod sentence_split;
pub mod sanitizer;

pub use correction::VoiceCorrectionEngine;
