// 모델 그룹 라우팅 — 여러 모델을 그룹화하여 라운드로빈 또는 폴백
// 원본: ModelGroup.kt, ModelGroupRouter.kt

use serde::{Deserialize, Serialize};
use crate::types::ModelEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelGroup {
    pub id: String,
    pub name: String,
    pub model_ids: Vec<String>,
    pub strategy: GroupStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GroupStrategy {
    #[default]
    RoundRobin,  // 순차적으로 모델 순회
    Fallback,    // 첫 번째 모델 실패 시 다음 모델
    Primary,     // 항상 첫 번째 모델만 사용
}

impl ModelGroup {
    pub fn select_model(&self, index: usize) -> Option<&String> {
        if self.model_ids.is_empty() {
            return None;
        }
        match self.strategy {
            GroupStrategy::RoundRobin => {
                Some(&self.model_ids[index % self.model_ids.len()])
            }
            GroupStrategy::Fallback | GroupStrategy::Primary => {
                self.model_ids.first()
            }
        }
    }
}