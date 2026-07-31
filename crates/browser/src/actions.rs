// 브라우저 액션 정의

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserActionInput {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinate_x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinate_y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserActionResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub minis_url: Option<String>,
}

pub enum BrowserAction {
    Navigate(String),
    Screenshot,
    Click { selector: Option<String>, x: Option<i32>, y: Option<i32> },
    Type { selector: String, text: String },
    Scroll { direction: String, amount: i32 },
    GetText { selector: String },
    GetReadable,
    ExecuteJs(String),
    FindElements(String),
    Fetch(String),
}
