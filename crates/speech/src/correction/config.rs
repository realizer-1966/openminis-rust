// 음성 교정 설정 — 모든 튜닝 파라미터
// 원본: VoiceCorrectionConfig.kt

pub struct VoiceCorrectionConfig;

impl VoiceCorrectionConfig {
    /// 학습된 쌍의 최소 신뢰도
    pub const MIN_CONFUSION_CONFIDENCE: f64 = 0.3;

    /// 프롬프트에 포함할 최대 후보 수
    pub const MAX_CANDIDATES: u32 = 20;

    /// 검색 단계 예산 (ms)
    pub const RETRIEVAL_BUDGET_MS: u64 = 80;

    /// LLM 호출 타임아웃 (ms) — 느린 프로바이더도 수용
    pub const CORRECTION_BUDGET_MS: u64 = 15_000;

    /// 문자 변경 비율 상한 — 이 이상이면 재작성으로 판단
    pub const MAX_CHAR_CHANGE_RATIO: f64 = 0.5;

    /// 용어 최소 출현 빈도
    pub const VOCAB_MIN_FREQUENCY: u32 = 3;

    /// 용어 최소 등장 일수
    pub const VOCAB_MIN_DISTINCT_DAYS: u32 = 2;

    /// 어휘 승인 최소 점수
    pub const VOCAB_MIN_SCORE: u32 = 2;

    /// 배경 빈도 순위 임계값 — 이 위는 rare
    pub const VOCAB_BACKGROUND_RANK_THRESHOLD: u32 = 3000;

    /// 후보 캐시 TTL (ms)
    pub const CACHE_TTL_MS: u64 = 300_000;

    /// 후보 캐시 최대 항목 수
    pub const CACHE_LIMIT: usize = 200;

    /// 음성 유사도 임계값
    pub const PHONETIC_SIMILARITY_THRESHOLD: f64 = 0.6;

    /// 컨텍스트 샘플 문자 수
    pub const CONTEXT_SAMPLE_CHARS: usize = 50;
}