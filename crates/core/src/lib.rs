// minis-core: 에이전트 코어 — 툴 루프, 툴 디스패치, 세션 관리
// OpenMinis Android의 agent/ + tools/ 디렉토리를 Rust로 이관

pub mod agent_loop;
pub mod tool_dispatch;
pub mod tools;
pub mod session;
pub mod soul;
pub mod compaction;
pub mod mock_provider;

#[cfg(target_os = "android")]
mod jni_android;

#[cfg(test)]
mod agent_loop_tests;

pub use agent_loop::AgentLoop;
pub use session::Session;
pub use tool_dispatch::ToolDispatcher;
pub use compaction::{compact_messages, needs_compaction, CompactionConfig};