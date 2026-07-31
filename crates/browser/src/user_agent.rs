// User-Agent 프로파일 — 모바일/데스크톱 Chrome 에뮬레이션
// 원본: BrowserAction.kt의 UserAgentProfile enum

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserAgentProfile {
    MobileChrome,
    DesktopChrome,
    Custom,
}

impl UserAgentProfile {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "mobile_chrome" => Self::MobileChrome,
            "desktop_chrome" => Self::DesktopChrome,
            "custom" => Self::Custom,
            _ => return None,
        })
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MobileChrome => "mobile_chrome",
            Self::DesktopChrome => "desktop_chrome",
            Self::Custom => "custom",
        }
    }

    pub fn user_agent_string(&self) -> Option<&'static str> {
        match self {
            Self::MobileChrome => Some("Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Mobile Safari/537.36"),
            Self::DesktopChrome => Some("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36"),
            Self::Custom => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::MobileChrome => "Mobile Chrome",
            Self::DesktopChrome => "Desktop Chrome",
            Self::Custom => "Custom",
        }
    }

    pub fn viewport_size(&self) -> (i32, i32) {
        match self {
            Self::DesktopChrome => (1280, 800),
            Self::MobileChrome | Self::Custom => (412, 915),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(UserAgentProfile::from_str("mobile_chrome"), Some(UserAgentProfile::MobileChrome));
        assert_eq!(UserAgentProfile::from_str("desktop_chrome"), Some(UserAgentProfile::DesktopChrome));
        assert_eq!(UserAgentProfile::from_str("invalid"), None);
    }

    #[test]
    fn test_user_agent_string() {
        assert!(UserAgentProfile::MobileChrome.user_agent_string().unwrap().contains("Android"));
        assert!(UserAgentProfile::DesktopChrome.user_agent_string().unwrap().contains("Linux x86_64"));
        assert_eq!(UserAgentProfile::Custom.user_agent_string(), None);
    }

    #[test]
    fn test_viewport_size() {
        assert_eq!(UserAgentProfile::DesktopChrome.viewport_size(), (1280, 800));
        assert_eq!(UserAgentProfile::MobileChrome.viewport_size(), (412, 915));
    }
}