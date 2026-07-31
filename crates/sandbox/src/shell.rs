// PersistentShell — PTY 기반 지속 셸 세션
// 원본: PersistentShell.kt, ShellExecutor.kt

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
    pub duration_ms: u64,
    /// 실행 후 /var/minis/에 새로 생성된 파일의 minis:// URL
    pub new_minis_urls: Vec<String>,
}

pub struct PersistentShell {
    // TODO: PTY 브릿지 (pty_bridge.c FFI)
}

impl PersistentShell {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn execute(&self, _command: &str, _timeout: Duration) -> Result<ShellResult> {
        // TODO: PTY에 명령 작성, 출력 읽기, 타임아웃 처리
        Ok(ShellResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            timed_out: false,
            duration_ms: 0,
            new_minis_urls: vec![],
        })
    }

    pub async fn is_alive(&self) -> bool {
        true
    }
}

impl Default for PersistentShell {
    fn default() -> Self { Self::new() }
}
