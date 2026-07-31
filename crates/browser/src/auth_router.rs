// Google OAuth 라우터 — Google 로그인 도메인을 시스템 브라우저로 우회
// 원본: GoogleAuthRouter.kt
//
// Google은 WebView 기반 로그인을 영구 차단하므로,
// accounts.google.com 등의 도메인은 Chrome Custom Tab으로 우회해야 함.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAuthRouter;

impl GoogleAuthRouter {
    /// Google 인증 도메인 목록 (서브도메인 매칭)
    const GOOGLE_AUTH_HOSTS: &'static [&'static str] = &[
        "accounts.google.com",
        "signin.google.com",
        "myaccount.google.com",
        "oauth2.googleapis.com",
        "accounts.youtube.com",
    ];

    /// URL이 Google 로그인 도메인인지 확인
    pub fn is_google_auth_url(url: &str) -> bool {
        let host = match Self::extract_host(url) {
            Some(h) => h.to_lowercase(),
            None => return false,
        };
        Self::GOOGLE_AUTH_HOSTS.iter().any(|auth_host| {
            host == *auth_host || host.ends_with(&format!(".{}", auth_host))
        })
    }

    /// URL에서 호스트 추출
    fn extract_host(url: &str) -> Option<&str> {
        let stripped = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);
        let host_end = stripped.find('/').unwrap_or(stripped.len());
        let host = &stripped[..host_end];
        // 포트 제거
        let host = host.split(':').next().unwrap_or(host);
        if host.is_empty() { None } else { Some(host) }
    }

    /// disallowed_useragent 또는 403 응답 감지
    pub fn is_disallowed_useragent(text: &str) -> bool {
        text.contains("disallowed_useragent")
            || text.contains("browser is not secure")
            || text.contains("403")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_auth_url() {
        assert!(GoogleAuthRouter::is_google_auth_url("https://accounts.google.com/signin"));
        assert!(GoogleAuthRouter::is_google_auth_url("https://signin.google.com/"));
        assert!(GoogleAuthRouter::is_google_auth_url("https://oauth2.googleapis.com/token"));
        assert!(GoogleAuthRouter::is_google_auth_url("https://sub.accounts.google.com/login"));
    }

    #[test]
    fn test_non_google_url() {
        assert!(!GoogleAuthRouter::is_google_auth_url("https://example.com"));
        assert!(!GoogleAuthRouter::is_google_auth_url("https://github.com/login"));
        assert!(!GoogleAuthRouter::is_google_auth_url("https://google.com/search"));
    }

    #[test]
    fn test_disallowed_useragent() {
        assert!(GoogleAuthRouter::is_disallowed_useragent("Error: disallowed_useragent"));
        assert!(GoogleAuthRouter::is_disallowed_useragent("403 browser is not secure"));
        assert!(!GoogleAuthRouter::is_disallowed_useragent("normal page content"));
    }

    #[test]
    fn test_extract_host() {
        assert_eq!(GoogleAuthRouter::extract_host("https://example.com/path"), Some("example.com"));
        assert_eq!(GoogleAuthRouter::extract_host("http://localhost:8080"), Some("localhost"));
        assert_eq!(GoogleAuthRouter::extract_host("not-a-url"), Some("not-a-url"));
    }
}