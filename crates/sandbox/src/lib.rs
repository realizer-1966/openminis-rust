// 샌드박스 — PRoot 기반 Linux 환경 관리
// 원본: sandbox/ 디렉토리 (PRootKernel.kt, PersistentShell.kt, ShellExecutor.kt, RootfsManager.kt)

pub mod proot;
pub mod shell;
pub mod rootfs;
pub mod bashism;
pub mod terminal;

pub use shell::PersistentShell;
pub use bashism::BashismDetector;
