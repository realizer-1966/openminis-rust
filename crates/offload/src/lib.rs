// Offload 시스템 — Android 기능 위임 인터페이스
// 원본: sandbox/offload/ (25개 핸들러), offload/ (13개 매니저)
//
// 핵심 설계:
// - Rust: 요청 파싱, 권한 게이트, 인자 검증, 응답 포맷팅
// - JNI: 실제 Android API 호출 (ContentResolver, AlarmManager, etc.)
// - Weather는 Rust에서 직접 HTTP 호출 (JNI 불필요)

pub mod trait_def;
pub mod registry;
pub mod args;
pub mod gate;
pub mod handlers;

#[cfg(test)]
mod tests;

pub use trait_def::{OffloadHandler, OffloadResult, PermissionState};
pub use registry::OffloadRegistry;
pub use args::OffloadArgs;
pub use gate::OffloadGate;

// 핸들러 re-exports
pub use handlers::calendar::CalendarHandler;
pub use handlers::contacts::ContactsHandler;
pub use handlers::alarm::AlarmHandler;
pub use handlers::location::LocationHandler;
pub use handlers::weather::WeatherHandler;
pub use handlers::device::DeviceHandler;
pub use handlers::notification::NotificationHandler;
pub use handlers::photos::PhotosHandler;
pub use handlers::clipboard::ClipboardHandler;
pub use handlers::speech::{SpeakHandler, SpeechHandler};
pub use handlers::shizuku::ShizukuHandler;