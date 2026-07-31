// 스킬 매칭 — 사용자 요청에 맞는 스킬 찾기
// 원본: SessionSkillsSheet.kt, AgentTools의 스킬 로딩 부분
//
// 매칭 전략:
// 1. Trigger exact match (우선순위 최고)
// 2. Keyword fuzzy match
// 3. 설명 텍스트 단어 매칭
// 점수 기반 정렬 — 여러 스킬이 매칭되면 점수 순

use crate::loader::Skill;

/// 매칭 결과 — 스킬과 점수
#[derive(Debug, Clone)]
pub struct SkillMatch {
    pub skill: Skill,
    pub score: f32,
    pub matched_by: MatchType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    TriggerExact,
    TriggerSubstring,
    KeywordMatch,
    DescriptionMatch,
}

impl MatchType {
    fn weight(&self) -> f32 {
        match self {
            MatchType::TriggerExact => 1.0,
            MatchType::TriggerSubstring => 0.8,
            MatchType::KeywordMatch => 0.5,
            MatchType::DescriptionMatch => 0.3,
        }
    }
}

pub struct SkillMatcher {
    skills: Vec<Skill>,
}

impl SkillMatcher {
    pub fn new(skills: Vec<Skill>) -> Self {
        Self { skills }
    }

    /// 요청과 매칭되는 스킬을 점수순으로 반환
    pub fn match_request(&self, request: &str) -> Vec<SkillMatch> {
        let request_lower = request.to_lowercase();
        let mut matches: Vec<SkillMatch> = Vec::new();

        for skill in &self.skills {
            let mut best_score = 0.0f32;
            let mut best_match = MatchType::DescriptionMatch;

            // 1. Trigger exact/substring match
            for trigger in &skill.metadata.triggers {
                let trigger_lower = trigger.to_lowercase();
                if request_lower == trigger_lower {
                    best_score = best_score.max(MatchType::TriggerExact.weight());
                    best_match = MatchType::TriggerExact;
                } else if request_lower.contains(&trigger_lower) {
                    let score = MatchType::TriggerSubstring.weight() * (trigger_lower.len() as f32 / request_lower.len().max(1) as f32);
                    if score > best_score {
                        best_score = score;
                        best_match = MatchType::TriggerSubstring;
                    }
                }
            }

            // 2. Keyword match
            for keyword in &skill.metadata.keywords {
                let kw_lower = keyword.to_lowercase();
                if request_lower.contains(&kw_lower) {
                    let score = MatchType::KeywordMatch.weight();
                    if score > best_score {
                        best_score = score;
                        best_match = MatchType::KeywordMatch;
                    }
                }
            }

            // 3. Description word match (always lower than keyword)
            if best_score < MatchType::KeywordMatch.weight() {
                for word in skill.description.split_whitespace() {
                    let word_lower = word.to_lowercase();
                    if word_lower.len() >= 3 && request_lower.contains(&word_lower) {
                        let score = MatchType::DescriptionMatch.weight();
                        if score > best_score {
                            best_score = score;
                            best_match = MatchType::DescriptionMatch;
                        }
                    }
                }
            }

            if best_score > 0.0 {
                matches.push(SkillMatch {
                    skill: skill.clone(),
                    score: best_score,
                    matched_by: best_match,
                });
            }
        }

        // 점수순 정렬
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    /// 상위 N개 매칭 스킬만 반환
    pub fn match_top(&self, request: &str, n: usize) -> Vec<SkillMatch> {
        let mut matches = self.match_request(request);
        matches.truncate(n);
        matches
    }

    /// 매칭 스킬 수
    pub fn count(&self) -> usize {
        self.skills.len()
    }

    /// 스킬 이름으로 조회
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::{Skill, SkillMetadata};
    use std::path::PathBuf;

    fn make_skill(name: &str, triggers: Vec<&str>, keywords: Vec<&str>, desc: &str) -> Skill {
        Skill {
            name: name.into(),
            description: desc.into(),
            path: PathBuf::from(format!("/skills/{}", name)),
            metadata: SkillMetadata {
                triggers: triggers.into_iter().map(String::from).collect(),
                keywords: keywords.into_iter().map(String::from).collect(),
                tools: vec![],
            },
            body: String::new(),
        }
    }

    #[test]
    fn test_exact_trigger_match() {
        let matcher = SkillMatcher::new(vec![
            make_skill("weather", vec!["weather forecast"], vec![], "Get weather info"),
        ]);
        let matches = matcher.match_request("weather forecast");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_by, MatchType::TriggerExact);
    }

    #[test]
    fn test_keyword_match() {
        let matcher = SkillMatcher::new(vec![
            make_skill("tts", vec![], vec!["voice", "speech", "audio"], "Text to speech"),
        ]);
        let matches = matcher.match_request("convert text to voice output");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_by, MatchType::KeywordMatch);
    }

    #[test]
    fn test_no_match() {
        let matcher = SkillMatcher::new(vec![
            make_skill("weather", vec!["weather"], vec!["rain"], "Weather info"),
        ]);
        let matches = matcher.match_request("make me a sandwich");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_score_ordering() {
        let matcher = SkillMatcher::new(vec![
            make_skill("low", vec![], vec!["data"], "data analysis"),
            make_skill("high", vec!["analyze data"], vec!["data"], "Data analysis tool"),
        ]);
        let matches = matcher.match_request("analyze data");
        assert_eq!(matches.len(), 2);
        // "high" has trigger match → higher score
        assert_eq!(matches[0].skill.name, "high");
    }

    #[test]
    fn test_match_top() {
        let matcher = SkillMatcher::new(vec![
            make_skill("a", vec!["test"], vec![], "a"),
            make_skill("b", vec!["test"], vec![], "b"),
            make_skill("c", vec!["test"], vec![], "c"),
        ]);
        let matches = matcher.match_top("test", 2);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_get_by_name() {
        let matcher = SkillMatcher::new(vec![
            make_skill("my-skill", vec![], vec![], "test"),
        ]);
        assert!(matcher.get("my-skill").is_some());
        assert!(matcher.get("nope").is_none());
    }
}