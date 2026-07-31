// SKILL.md 파싱/로드

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub metadata: SkillMetadata,
    pub body: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub triggers: Vec<String>,
    pub keywords: Vec<String>,
    pub tools: Vec<String>,
}

pub struct SkillLoader;

impl SkillLoader {
    pub fn load_skill(skill_dir: &std::path::Path) -> anyhow::Result<Skill> {
        let skill_md = skill_dir.join("SKILL.md");
        let content = std::fs::read_to_string(&skill_md)?;

        // 간단한 파싱 — frontmatter와 body 분리
        let (metadata, body) = if content.starts_with("---") {
            let end = content[3..].find("---")
                .map(|pos| pos + 3);
            if let Some(end) = end {
                let frontmatter = &content[3..end];
                let body = content[end+3..].trim().to_string();
                (parse_frontmatter(frontmatter), body)
            } else {
                (SkillMetadata::default(), content)
            }
        } else {
            (SkillMetadata::default(), content)
        };

        let name = skill_dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Skill {
            name,
            description: metadata.keywords.first().cloned().unwrap_or_default(),
            path: skill_dir.to_path_buf(),
            metadata,
            body,
        })
    }

    pub fn load_all(skills_dir: &std::path::Path) -> Vec<Skill> {
        let mut skills = Vec::new();
        if let Ok(entries) = std::fs::read_dir(skills_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Ok(skill) = Self::load_skill(&entry.path()) {
                        skills.push(skill);
                    }
                }
            }
        }
        skills
    }
}

fn parse_frontmatter(text: &str) -> SkillMetadata {
    let mut triggers = Vec::new();
    let mut keywords = Vec::new();
    let mut tools = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("triggers:") {
            triggers = rest.split(',').map(|s| s.trim().to_string()).collect();
        } else if let Some(rest) = line.strip_prefix("keywords:") {
            keywords = rest.split(',').map(|s| s.trim().to_string()).collect();
        } else if let Some(rest) = line.strip_prefix("tools:") {
            tools = rest.split(',').map(|s| s.trim().to_string()).collect();
        }
    }

    SkillMetadata { triggers, keywords, tools }
}
