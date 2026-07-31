// Bashism 감지 — BusyBox ash와 bash의 차이점 감지
// 원본: src/shared/bashism/bashism_rules.json, agent/shell/BashismDetector.kt

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashismRule {
    pub pattern: String,
    pub message: String,
    pub suggestion: Option<String>,
}

pub struct BashismDetector {
    rules: Vec<(Regex, BashismRule)>,
}

impl BashismDetector {
    pub fn new() -> Self {
        let rules_json = include_str!("bashism_rules.json");
        let rules: Vec<BashismRule> = serde_json::from_str(rules_json)
            .unwrap_or_default();
        let compiled: Vec<(Regex, BashismRule)> = rules
            .into_iter()
            .filter_map(|r| Regex::new(&r.pattern).ok().map(|re| (re, r)))
            .collect();
        Self { rules: compiled }
    }

    pub fn detect(&self, command: &str) -> Vec<&BashismRule> {
        self.rules
            .iter()
            .filter(|(re, _)| re.is_match(command))
            .map(|(_, rule)| rule)
            .collect()
    }

    pub fn has_bashisms(&self, command: &str) -> bool {
        !self.detect(command).is_empty()
    }
}

impl Default for BashismDetector {
    fn default() -> Self { Self::new() }
}
