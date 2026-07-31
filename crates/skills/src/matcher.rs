// 스킬 매칭 — 사용자 요청에 맞는 스킬 찾기

use crate::loader::Skill;

pub struct SkillMatcher {
    skills: Vec<Skill>,
}

impl SkillMatcher {
    pub fn new(skills: Vec<Skill>) -> Self {
        Self { skills }
    }

    pub fn match_request(&self, request: &str) -> Vec<&Skill> {
        let request_lower = request.to_lowercase();
        self.skills
            .iter()
            .filter(|skill| {
                skill.metadata.triggers.iter().any(|t| request_lower.contains(&t.to_lowercase()))
                    || skill.metadata.keywords.iter().any(|k| request_lower.contains(&k.to_lowercase()))
            })
            .collect()
    }
}
