// Bashism 감지 — BusyBox ash와 bash의 차이점 감지
// 원본: src/shared/bashism/bashism_rules.json, agent/shell/BashismDetector.kt

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashismRule {
    pub name: String,
    pub pattern: String,
    #[serde(rename = "behaviorNote")]
    pub behavior_note: Option<String>,
    #[serde(rename = "fixHint")]
    pub fix_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BashismRulesFile {
    rules: Vec<BashismRule>,
}

pub struct BashismDetector {
    rules: Vec<(Regex, BashismRule)>,
}

impl BashismDetector {
    pub fn new() -> Self {
        let rules_json = include_str!("bashism_rules.json");
        let rules_file: BashismRulesFile = serde_json::from_str(rules_json)
            .unwrap_or(BashismRulesFile { rules: vec![] });
        let compiled: Vec<(Regex, BashismRule)> = rules_file.rules
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loads_rules() {
        let detector = BashismDetector::new();
        assert!(!detector.rules.is_empty(), "Should load rules from JSON");
    }

    #[test]
    fn test_bash_shebang() {
        let detector = BashismDetector::new();
        assert!(detector.has_bashisms("#!/bin/bash\necho hi"));
    }

    #[test]
    fn test_array_bashism() {
        let detector = BashismDetector::new();
        // arr=(1 2 3) is a bashism — array assignment
        let result = detector.detect("arr=(1 2 3); echo ${arr[@]}");
        assert!(!result.is_empty(), "Array assignment should be detected");
    }

    #[test]
    fn test_no_bashism() {
        let detector = BashismDetector::new();
        assert!(!detector.has_bashisms("echo hello && ls -la"));
        assert!(!detector.has_bashisms("for i in 1 2 3; do echo $i; done"));
    }
}
