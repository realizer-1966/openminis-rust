// 브라우저 액션 정의 — 모든 액션 열거형, 입력/출력 구조체
// 원본: BrowserAction.kt, BrowserActionInput.kt, BrowserActionResult.kt

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 모든 브라우저 액션
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrowserAction {
    Navigate,
    Screenshot,
    Click,
    Type,
    GetText,
    Scroll,
    GetPageInfo,
    ExecuteJs,
    FindElements,
    Hover,
    GetReadable,
    SetUserAgent,
    SetViewport,
    GetBackbone,
    Fetch,
    NewTab,
    CloseTab,
    ListTabs,
    GetCookies,
    SetCookies,
    ScrollAndCollect,
    WaitForDomStable,
}

impl BrowserAction {
    /// 액션 문자열에서 변환
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "navigate" => Self::Navigate,
            "screenshot" => Self::Screenshot,
            "click" => Self::Click,
            "type" => Self::Type,
            "get_text" => Self::GetText,
            "scroll" => Self::Scroll,
            "get_page_info" => Self::GetPageInfo,
            "execute_js" => Self::ExecuteJs,
            "find_elements" => Self::FindElements,
            "hover" => Self::Hover,
            "get_readable" => Self::GetReadable,
            "set_user_agent" => Self::SetUserAgent,
            "set_viewport" => Self::SetViewport,
            "get_backbone" => Self::GetBackbone,
            "fetch" => Self::Fetch,
            "new_tab" => Self::NewTab,
            "close_tab" => Self::CloseTab,
            "list_tabs" => Self::ListTabs,
            "get_cookies" => Self::GetCookies,
            "set_cookies" => Self::SetCookies,
            "scroll_and_collect" => Self::ScrollAndCollect,
            "wait_for_dom_stable" => Self::WaitForDomStable,
            _ => return None,
        })
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Navigate => "navigate",
            Self::Screenshot => "screenshot",
            Self::Click => "click",
            Self::Type => "type",
            Self::GetText => "get_text",
            Self::Scroll => "scroll",
            Self::GetPageInfo => "get_page_info",
            Self::ExecuteJs => "execute_js",
            Self::FindElements => "find_elements",
            Self::Hover => "hover",
            Self::GetReadable => "get_readable",
            Self::SetUserAgent => "set_user_agent",
            Self::SetViewport => "set_viewport",
            Self::GetBackbone => "get_backbone",
            Self::Fetch => "fetch",
            Self::NewTab => "new_tab",
            Self::CloseTab => "close_tab",
            Self::ListTabs => "list_tabs",
            Self::GetCookies => "get_cookies",
            Self::SetCookies => "set_cookies",
            Self::ScrollAndCollect => "scroll_and_collect",
            Self::WaitForDomStable => "wait_for_dom_stable",
        }
    }

    /// 페이지를 새로 열거나 교체하는 액션 (fan-out 대상)
    pub fn opens_new_page(&self) -> bool {
        matches!(self, Self::Navigate)
    }

    /// 시각적 변경을 일으키는 액션 (스크린샷 자동 캡처 대상)
    pub fn is_visual_change(&self) -> bool {
        matches!(self, Self::Navigate | Self::Click | Self::Scroll | Self::Hover | Self::Type)
    }

    /// 모든 액션 문자열 목록
    pub fn all_values() -> Vec<&'static str> {
        vec![
            "navigate", "screenshot", "click", "type", "get_text", "scroll",
            "get_page_info", "execute_js", "find_elements", "hover",
            "get_readable", "set_user_agent", "set_viewport", "get_backbone",
            "fetch", "new_tab", "close_tab", "list_tabs", "get_cookies",
            "set_cookies", "scroll_and_collect", "wait_for_dom_stable",
        ]
    }
}

/// 스크롤 방향
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
}

impl ScrollDirection {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
        }
    }
}

/// 브라우저 액션 입력 — JSON에서 파싱
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrowserActionInput {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinate_x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinate_y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport_width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport_height: Option<i32>,
    #[serde(default)]
    pub reset: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    #[serde(default)]
    pub fuzzy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i32>,
    #[serde(default)]
    pub full_page: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<Vec<Value>>,
}

impl BrowserActionInput {
    /// JSON 문자열에서 파싱
    pub fn parse(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }

    /// 액션 enum으로 변환
    pub fn browser_action(&self) -> Option<BrowserAction> {
        BrowserAction::from_str(&self.action)
    }

    /// 필수 필드 검증
    pub fn validate(&self) -> Result<(), String> {
        let action = self.browser_action()
            .ok_or_else(|| format!("Unknown action: {}", self.action))?;

        match action {
            BrowserAction::Navigate => {
                if self.url.is_none() {
                    return Err("navigate: url is required".into());
                }
            }
            BrowserAction::Type => {
                if self.selector.is_none() && (self.coordinate_x.is_none() || self.coordinate_y.is_none()) {
                    return Err("type: selector or coordinates required".into());
                }
                if self.text.is_none() {
                    return Err("type: text is required".into());
                }
            }
            BrowserAction::Click => {
                if self.selector.is_none() && (self.coordinate_x.is_none() || self.coordinate_y.is_none()) {
                    return Err("click: selector or coordinates required".into());
                }
            }
            BrowserAction::Scroll => {
                if self.direction.is_none() {
                    return Err("scroll: direction is required".into());
                }
            }
            BrowserAction::ExecuteJs => {
                if self.script.is_none() {
                    return Err("execute_js: script is required".into());
                }
            }
            BrowserAction::Fetch => {
                if self.url.is_none() {
                    return Err("fetch: url is required".into());
                }
            }
            BrowserAction::ScrollAndCollect => {
                if self.item_selector.is_none() {
                    return Err("scroll_and_collect: item_selector is required".into());
                }
            }
            _ => {}
        }

        Ok(())
    }
}

/// 브라우저 액션 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserActionResult {
    pub text: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base64_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minis_url: Option<String>,
}

impl BrowserActionResult {
    pub fn ok(text: String) -> Self {
        Self { text, success: true, base64_image: None, image_file_path: None, page_url: None, tab_id: None, minis_url: None }
    }

    pub fn error(message: &str) -> Self {
        Self { text: format!("Error: {}", message), success: false, base64_image: None, image_file_path: None, page_url: None, tab_id: None, minis_url: None }
    }

    pub fn screenshot(text: String, base64: String, file_path: String, minis_url: String) -> Self {
        Self {
            text,
            success: true,
            base64_image: Some(base64),
            image_file_path: Some(file_path),
            page_url: None,
            tab_id: None,
            minis_url: Some(minis_url),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_from_str() {
        assert_eq!(BrowserAction::from_str("navigate"), Some(BrowserAction::Navigate));
        assert_eq!(BrowserAction::from_str("screenshot"), Some(BrowserAction::Screenshot));
        assert_eq!(BrowserAction::from_str("invalid"), None);
    }

    #[test]
    fn test_action_as_str() {
        assert_eq!(BrowserAction::Navigate.as_str(), "navigate");
        assert_eq!(BrowserAction::ScrollAndCollect.as_str(), "scroll_and_collect");
    }

    #[test]
    fn test_opens_new_page() {
        assert!(BrowserAction::Navigate.opens_new_page());
        assert!(!BrowserAction::Click.opens_new_page());
        assert!(!BrowserAction::Screenshot.opens_new_page());
    }

    #[test]
    fn test_visual_change() {
        assert!(BrowserAction::Navigate.is_visual_change());
        assert!(BrowserAction::Click.is_visual_change());
        assert!(!BrowserAction::GetText.is_visual_change());
    }

    #[test]
    fn test_all_values() {
        let vals = BrowserAction::all_values();
        assert_eq!(vals.len(), 22);
        assert!(vals.contains(&"navigate"));
        assert!(vals.contains(&"wait_for_dom_stable"));
    }

    #[test]
    fn test_scroll_direction() {
        assert_eq!(ScrollDirection::from_str("up"), Some(ScrollDirection::Up));
        assert_eq!(ScrollDirection::from_str("down"), Some(ScrollDirection::Down));
        assert_eq!(ScrollDirection::from_str("sideways"), None);
    }

    #[test]
    fn test_input_parse() {
        let json = r#"{"action":"navigate","url":"https://example.com"}"#;
        let input = BrowserActionInput::parse(json).unwrap();
        assert_eq!(input.action, "navigate");
        assert_eq!(input.url.as_deref(), Some("https://example.com"));
    }

    #[test]
    fn test_input_validate_navigate_ok() {
        let input = BrowserActionInput {
            action: "navigate".into(),
            url: Some("https://example.com".into()),
            ..Default::default()
        };
        assert!(input.validate().is_ok());
    }

    #[test]
    fn test_input_validate_navigate_missing_url() {
        let input = BrowserActionInput {
            action: "navigate".into(),
            ..Default::default()
        };
        assert!(input.validate().is_err());
    }

    #[test]
    fn test_input_validate_type_missing_text() {
        let input = BrowserActionInput {
            action: "type".into(),
            selector: Some("#input".into()),
            ..Default::default()
        };
        assert!(input.validate().is_err());
    }

    #[test]
    fn test_input_validate_scroll_missing_direction() {
        let input = BrowserActionInput {
            action: "scroll".into(),
            ..Default::default()
        };
        assert!(input.validate().is_err());
    }

    #[test]
    fn test_input_validate_unknown_action() {
        let input = BrowserActionInput {
            action: "frobnicate".into(),
            ..Default::default()
        };
        assert!(input.validate().is_err());
    }

    #[test]
    fn test_input_validate_click_with_coords() {
        let input = BrowserActionInput {
            action: "click".into(),
            coordinate_x: Some(100),
            coordinate_y: Some(200),
            ..Default::default()
        };
        assert!(input.validate().is_ok());
    }

    #[test]
    fn test_result_ok() {
        let r = BrowserActionResult::ok("done".into());
        assert!(r.success);
        assert_eq!(r.text, "done");
    }

    #[test]
    fn test_result_error() {
        let r = BrowserActionResult::error("failed");
        assert!(!r.success);
        assert!(r.text.contains("failed"));
    }

    #[test]
    fn test_result_screenshot() {
        let r = BrowserActionResult::screenshot("captured".into(), "base64".into(), "/tmp/shot.jpg".into(), "minis://browser/shot.jpg".into());
        assert!(r.success);
        assert_eq!(r.minis_url.as_deref(), Some("minis://browser/shot.jpg"));
        assert!(r.base64_image.is_some());
    }
}