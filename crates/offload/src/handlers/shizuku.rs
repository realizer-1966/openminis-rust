// Shizuku offload — 권한이 필요한 Android 시스템 API 호출
// 원본: ShizukuOffloadHandler.kt, ShizukuBackend.kt, ShizukuManager.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::trait_def::{OffloadHandler, OffloadResult, PermissionState};

pub struct ShizukuHandler {
    jni_callback: Option<Box<dyn Fn(&str, &Value) -> Result<Value> + Send + Sync>>,
}

impl ShizukuHandler {
    pub fn new() -> Self { Self { jni_callback: None } }
    pub fn with_jni<F>(callback: F) -> Self
    where F: Fn(&str, &Value) -> Result<Value> + Send + Sync + 'static {
        Self { jni_callback: Some(Box::new(callback)) }
    }
}

const HELP: &str = r#"android-shizuku-cli — invoke privileged Android system APIs via Shizuku.

Usage:
  android-shizuku-cli exec <shell command>          Run shell command with Shizuku privilege
  android-shizuku-cli <subcommand>                  Curated subcommands (see --help)
"#;

#[async_trait]
impl OffloadHandler for ShizukuHandler {
    fn name(&self) -> &str { "shizuku" }
    fn display_name(&self) -> &str { "android-shizuku-cli" }
    fn help(&self) -> &str { HELP }

    async fn execute(&self, args: Value) -> Result<OffloadResult> {
        let sub = args.get("subcommand")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if sub.is_empty() {
            return Ok(OffloadResult::bad_args("no subcommand specified"));
        }

        if let Some(cb) = &self.jni_callback {
            let result = cb(sub, &args)?;
            if result.get("error").is_some() {
                Ok(OffloadResult { success: false, data: result.clone(), error: result.get("error").and_then(|v| v.as_str()).map(|s| s.to_string()), exit_code: 1 })
            } else {
                Ok(OffloadResult::ok(result))
            }
        } else {
            Ok(OffloadResult::error("jni_callback not configured (Shizuku requires Android runtime)", 1))
        }
    }

    fn check_permission(&self, _session_id: Option<&str>) -> PermissionState {
        PermissionState::Allowed
    }
}

impl Default for ShizukuHandler { fn default() -> Self { Self::new() } }