// 키워드 검색 — 모든 메모리 파일에서 퍼지 매칭
// 원본: memory_get 기능

use anyhow::Result;
use std::path::{Path, PathBuf};

/// 검색 결과 항목
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub file: String,
    pub line: String,
    pub line_number: usize,
    pub context_before: String,
    pub context_after: String,
    pub match_count: u32,
}

/// 메모리 검색 — 모든 .md 파일에서 키워드 매칭
/// 모든 키워드가 포함된 라인을 찾고, 컨텍스트와 함께 반환
pub fn search_memory(memory_dir: &Path, keywords: &[String]) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();
    if !memory_dir.exists() {
        return Ok(results);
    }

    let mut files = Vec::new();
    collect_md_files(memory_dir, &mut files);
    files.sort();

    for file_path in &files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let line_lower = line.to_lowercase();
            let matched = keywords.iter()
                .filter(|k| !k.is_empty())
                .filter(|k| line_lower.contains(&k.to_lowercase()))
                .count();

            if matched > 0 && matched == keywords.iter().filter(|k| !k.is_empty()).count() {
                let context_before = if i > 0 { lines[i - 1].to_string() } else { String::new() };
                let context_after = if i + 1 < lines.len() { lines[i + 1].to_string() } else { String::new() };

                results.push(SearchResult {
                    file: filename.clone(),
                    line: line.to_string(),
                    line_number: i + 1,
                    context_before,
                    context_after,
                    match_count: matched as u32,
                });
            }
        }
    }

    // 파일명 역순 (최신 날짜 먼저), 라인 번호 순
    results.sort_by(|a, b| b.file.cmp(&a.file).then(a.line_number.cmp(&b.line_number)));
    Ok(results)
}

fn collect_md_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_md_files(&path, files);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                files.push(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("minis-mem-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_search_finds_match() {
        let dir = temp_dir();
        fs::write(
            dir.join("2026-01-01.md"),
            "# Log\nSome notes about Rust programming here\nMore stuff",
        ).unwrap();

        let results = search_memory(&dir, &["Rust".into()]).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].line.contains("Rust"));
        assert_eq!(results[0].line_number, 2);
    }

    #[test]
    fn test_search_multi_keyword() {
        let dir = temp_dir();
        fs::write(
            dir.join("2026-01-01.md"),
            "line1\nRust is great for systems\nRust and Python\n",
        ).unwrap();

        // both keywords must match
        let results = search_memory(&dir, &["Rust".into(), "Python".into()]).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].line.contains("Python"));
    }

    #[test]
    fn test_search_case_insensitive() {
        let dir = temp_dir();
        fs::write(dir.join("test.md"), "Learn RUST programming\n").unwrap();

        let results = search_memory(&dir, &["rust".into()]).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_no_match() {
        let dir = temp_dir();
        fs::write(dir.join("test.md"), "nothing relevant here\n").unwrap();

        let results = search_memory(&dir, &["quantum".into()]).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_empty_keywords() {
        let dir = temp_dir();
        fs::write(dir.join("test.md"), "some content\n").unwrap();

        let results = search_memory(&dir, &["".into()]).unwrap();
        assert!(results.is_empty()); // 빈 키워드는 무시
    }

    #[test]
    fn test_search_context() {
        let dir = temp_dir();
        fs::write(
            dir.join("test.md"),
            "line before\ntarget line here\nline after",
        ).unwrap();

        let results = search_memory(&dir, &["target".into()]).unwrap();
        assert_eq!(results[0].context_before, "line before");
        assert_eq!(results[0].context_after, "line after");
    }

    #[test]
    fn test_search_nonexistent_dir() {
        let results = search_memory(Path::new("/nonexistent/path"), &["test".into()]).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_multiple_files() {
        let dir = temp_dir();
        fs::write(dir.join("2026-01-02.md"), "recent Rust entry\n").unwrap();
        fs::write(dir.join("2026-01-01.md"), "older Rust entry\n").unwrap();

        let results = search_memory(&dir, &["Rust".into()]).unwrap();
        assert_eq!(results.len(), 2);
        // 최신 파일이 먼저 (역순 정렬)
        assert_eq!(results[0].file, "2026-01-02.md");
        assert_eq!(results[1].file, "2026-01-01.md");
    }
}