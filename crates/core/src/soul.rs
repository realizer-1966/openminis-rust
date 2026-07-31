// SOUL.md — 에이전트 성격/음성 설정
// 원본: SoulStore.kt

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Soul {
    pub name: String,
    pub style: String,
    pub lang: String,
    pub body: String,
}