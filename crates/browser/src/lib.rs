// 브라우저 자동화 로직 — 탭 풀, 쿠키, 히스토리, 액션 정의/검증
// 원본: browser/ 디렉토리 (8개 Kotlin 파일, 3900줄)
// 실제 WebView 제어는 offload経由 JNI → Kotlin

pub mod tab_pool;
pub mod cookie;
pub mod history;
pub mod actions;
pub mod auth_router;
pub mod user_agent;
pub mod js;

pub use actions::{BrowserAction, BrowserActionInput, BrowserActionResult, ScrollDirection};
pub use auth_router::GoogleAuthRouter;
pub use user_agent::UserAgentProfile;
pub use tab_pool::{TabPool, TabInfo};