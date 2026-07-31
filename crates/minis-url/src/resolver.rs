// URL 해석 — minis:// URL을 Linux 파일 경로로 변환

use anyhow::Result;
use std::path::PathBuf;
use crate::namespace::Namespace;
use crate::MinisUrlError;

pub struct UrlResolver;

impl UrlResolver {
    /// minis://attachments/photo.png → /var/minis/attachments/photo.png
    pub fn to_linux_path(url: &str) -> Result<PathBuf> {
        let rest = url.strip_prefix("minis://")
            .ok_or_else(|| MinisUrlError::InvalidUrl(url.to_string()))?;

        // 첫 번째 / 전까지가 namespace
        let (ns_str, path) = match rest.find('/') {
            Some(pos) => (&rest[..pos], &rest[pos..]),
            None => (rest, ""),
        };

        let ns = Namespace::from_str(ns_str)
            .ok_or_else(|| MinisUrlError::NamespaceNotFound(ns_str.to_string()))?;

        // 경로 순회 공격 방지
        if path.contains("..") {
            return Err(MinisUrlError::PathTraversal.into());
        }

        let mut full = PathBuf::from(ns.linux_prefix());
        if !path.is_empty() {
            full.push(path.trim_start_matches('/'));
        }
        Ok(full)
    }

    /// /var/minis/attachments/photo.png → minis://attachments/photo.png
    pub fn from_linux_path(path: &str) -> Option<String> {
        let prefix = "/var/minis/";
        if !path.starts_with(prefix) {
            return None;
        }
        let relative = &path[prefix.len()..];
        if relative.contains("..") {
            return None;
        }
        Some(format!("minis://{}", relative))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_linux_path() {
        let path = UrlResolver::to_linux_path("minis://attachments/photo.png").unwrap();
        assert_eq!(path, PathBuf::from("/var/minis/attachments/photo.png"));
    }

    #[test]
    fn test_from_linux_path() {
        let url = UrlResolver::from_linux_path("/var/minis/workspace/report.csv");
        assert_eq!(url, Some("minis://workspace/report.csv".into()));
    }

    #[test]
    fn test_path_traversal() {
        let result = UrlResolver::to_linux_path("minis://attachments/../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_path() {
        let path = UrlResolver::to_linux_path("minis://workspace/project/src/main.py").unwrap();
        assert_eq!(path, PathBuf::from("/var/minis/workspace/project/src/main.py"));
    }
}
