// 키워드 검색 — 모든 메모리 파일에서 퍼지 매칭

use anyhow::Result;
use std::path::PathBuf;

pub fn search_memory(memory_dir: &PathBuf, keywords: &[String]) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();
    if !memory_dir.exists() {
        return Ok(results);
    }

    for entry in std::fs::read_dir(memory_dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let filename = entry.file_name().to_string_lossy().to_string();
        let content = std::fs::read_to_string(entry.path())?;
        
        // 키워드 매칭 — 모든 키워드가 포함된 라인 찾기
        for line in content.lines() {
            let line_lower = line.to_lowercase();
            if keywords.iter().all(|k| line_lower.contains(&k.to_lowercase())) {
                results.push(SearchResult {
                    file: filename.clone(),
                    line: line.to_string(),
                    context: content.lines()
                        .take_while(|l| *l != line)
                        .last()
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }
    }

    Ok(results)
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file: String,
    pub line: String,
    pub context: String,
}
