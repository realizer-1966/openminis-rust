// PersistentShell — PRoot 환경에서 명령을 실행하는 지속 셸
// 원본: PersistentShell.kt, ShellExecutor.kt, PtyBridge.kt
//
// Android에서는 pty_bridge.c (JNI)를 통해 PTY를 열고,
// PRoot 환경(/bin/sh)에서 명령을 실행한다.
// 이 Rust 구현은:
// 1. 호스트 환경(Alpine/Linux)에서는 직접 프로세스 실행
// 2. Android 환경에서는 FFI로 pty_bridge 호출 (향후 구현)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::process::Command;
use tracing::{info, debug, warn};

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

/// 셸 실행 설정
#[derive(Debug, Clone)]
pub struct ShellConfig {
    /// 셸 실행 파일 경로 (기본: /bin/sh)
    pub shell_path: String,
    /// 작업 디렉토리
    pub working_dir: Option<PathBuf>,
    /// 환경 변수
    pub env: Vec<(String, String)>,
    /// /var/minis/ 경로 (minis:// URL 스캔용)
    pub minis_root: PathBuf,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            shell_path: "/bin/sh".into(),
            working_dir: None,
            env: vec![],
            minis_root: PathBuf::from("/var/minis"),
        }
    }
}

/// 지속 셸 — 매 명령마다 새 프로세스를 실행하되,
/// 환경 변수와 작업 디렉토리를 유지한다.
/// (향후 PTY 기반 지속 세션으로 업그레이드)
pub struct PersistentShell {
    config: ShellConfig,
}

impl PersistentShell {
    pub fn new() -> Self {
        Self { config: ShellConfig::default() }
    }

    pub fn with_config(config: ShellConfig) -> Self {
        Self { config }
    }

    /// 명령 실행
    pub async fn execute(&self, command: &str, timeout: Duration) -> Result<ShellResult> {
        let start = Instant::now();

        // 실행 전 /var/minis/ 파일 스냅샷
        let before_files = self.scan_minis_files();

        // /bin/sh -c "command" 실행
        let mut cmd = Command::new(&self.config.shell_path);
        cmd.arg("-c").arg(command);

        // 작업 디렉토리 설정
        if let Some(dir) = &self.config.working_dir {
            cmd.current_dir(dir);
        }

        // 환경 변수 설정
        for (key, value) in &self.config.env {
            cmd.env(key, value);
        }

        // stdout/stderr 캡처
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // 프로세스 시작
        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();
        let mut child = cmd.spawn()?;
        let stdout = tokio::time::timeout(timeout, child.stdout.take().unwrap().read_to_end(&mut stdout_buf)).await;
        let stderr = tokio::time::timeout(timeout, child.stderr.take().unwrap().read_to_end(&mut stderr_buf)).await;

        let timed_out = stdout.is_err() || stderr.is_err();
        if timed_out {
            // 타임아웃 — 프로세스 종료
            let _ = child.kill().await;
            warn!("Shell command timed out after {}s: {}", timeout.as_secs(), &command[..command.len().min(80)]);
        }

        let stdout_bytes = match stdout {
            Ok(Ok(_)) => stdout_buf,
            _ => Vec::new(),
        };
        let stderr_bytes = match stderr {
            Ok(Ok(_)) => stderr_buf,
            _ => Vec::new(),
        };

        let exit_status = if timed_out {
            // 타임아웃 시 exit code 124 (Linux timeout 관례)
            Some(124)
        } else {
            child.wait().await.ok().and_then(|s| s.code())
        };

        let stdout_str = String::from_utf8_lossy(&stdout_bytes).to_string();
        let stderr_str = String::from_utf8_lossy(&stderr_bytes).to_string();
        let duration_ms = start.elapsed().as_millis() as u64;

        // 실행 후 /var/minis/ 파일 스냅샷 — 새 파일 감지
        let after_files = self.scan_minis_files();
        let new_urls = self.diff_minis_urls(&before_files, &after_files);

        if !new_urls.is_empty() {
            debug!("Shell produced {} new minis:// files", new_urls.len());
        }

        Ok(ShellResult {
            stdout: stdout_str,
            stderr: stderr_str,
            exit_code: exit_status.unwrap_or(-1),
            timed_out,
            duration_ms,
            new_minis_urls: new_urls,
        })
    }

    /// 셸이 살아있는지 확인 (현재는 항상 true — 프로세스가 매번 새로 생성되므로)
    pub async fn is_alive(&self) -> bool {
        // /bin/sh 존재 여부로 확인
        Path::new(&self.config.shell_path).exists()
    }

    /// /var/minis/ 하위 파일 목록 스캔
    fn scan_minis_files(&self) -> HashSet<String> {
        let mut files = HashSet::new();
        self.scan_dir_recursive(&self.config.minis_root, &mut files);
        files
    }

    fn scan_dir_recursive(&self, dir: &Path, files: &mut HashSet<String>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.scan_dir_recursive(&path, files);
                } else if let Some(path_str) = path.to_str() {
                    files.insert(path_str.to_string());
                }
            }
        }
    }

    /// before/after 파일 차이를 minis:// URL로 변환
    fn diff_minis_urls(&self, before: &HashSet<String>, after: &HashSet<String>) -> Vec<String> {
        let prefix = self.config.minis_root.to_string_lossy().to_string();
        let mut urls = Vec::new();
        for path in after {
            if !before.contains(path) && path.starts_with(&prefix) {
                let relative = &path[prefix.len()..];
                let relative = relative.trim_start_matches('/');
                if !relative.contains("..") {
                    urls.push(format!("minis://{}", relative));
                }
            }
        }
        urls.sort();
        urls
    }
}

impl Default for PersistentShell {
    fn default() -> Self { Self::new() }
}

// tokio::io::AsyncReadExt 사용
use tokio::io::AsyncReadExt;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo() {
        let shell = PersistentShell::new();
        let result = shell.execute("echo hello", Duration::from_secs(5)).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn test_stderr() {
        let shell = PersistentShell::new();
        let result = shell.execute("echo error >&2", Duration::from_secs(5)).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.contains("error"));
    }

    #[tokio::test]
    async fn test_exit_code() {
        let shell = PersistentShell::new();
        let result = shell.execute("exit 42", Duration::from_secs(5)).await.unwrap();
        assert_eq!(result.exit_code, 42);
    }

    #[tokio::test]
    async fn test_timeout() {
        let shell = PersistentShell::new();
        let result = shell.execute("sleep 10", Duration::from_millis(100)).await.unwrap();
        assert!(result.timed_out);
        assert_eq!(result.exit_code, 124);
    }

    #[test]
    fn test_diff_urls() {
        let shell = PersistentShell::with_config(ShellConfig {
            minis_root: PathBuf::from("/var/minis"),
            ..Default::default()
        });
        let mut before = HashSet::new();
        before.insert("/var/minis/workspace/old.txt".into());
        let mut after = before.clone();
        after.insert("/var/minis/workspace/new.txt".into());
        after.insert("/var/minis/attachments/photo.png".into());

        let urls = shell.diff_minis_urls(&before, &after);
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"minis://workspace/new.txt".to_string()));
        assert!(urls.contains(&"minis://attachments/photo.png".to_string()));
    }
}