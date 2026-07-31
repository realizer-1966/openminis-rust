// Voice Activity Detection 인터페이스
// 원본: RealTimeCutVADLibrary (iOS), Android는 시스템 VAD 사용

pub trait VoiceActivityDetector: Send + Sync {
    fn detect(&self, samples: &[f32], sample_rate: u32) -> Vec<VadSegment>;
}

#[derive(Debug, Clone)]
pub struct VadSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub confidence: f32,
}
