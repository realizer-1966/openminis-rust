// 브라우저 자동화 로직 — 탭 풀, 쿠키, 히스토리
// 원본: browser/ 디렉토리
// 실제 WebView 제어는 offload経由で JNI → Kotlin

pub mod tab_pool;
pub mod cookie;
pub mod history;
pub mod actions;

pub use actions::BrowserAction;
