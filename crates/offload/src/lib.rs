// Offload 시스템 — Android 기능 위임 인터페이스
// 원본: sandbox/offload/ (25개 핸들러), offload/ (13개 매니저)

pub mod trait_def;
pub mod registry;

pub use trait_def::{OffloadHandler, OffloadResult};
pub use registry::OffloadRegistry;
