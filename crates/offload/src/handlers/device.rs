// 기기 정보 offload — 모델, OS 버전, 배터리, 저장공간
// 원본: DeviceOffloadHandler.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::trait_def::{OffloadHandler, OffloadResult, PermissionState};

pub struct DeviceHandler {
    jni_callback: Option<Box<dyn Fn(&str, &Value) -> Result<Value> + Send + Sync>>,
}

impl DeviceHandler {
    pub fn new() -> Self { Self { jni_callback: None } }
    pub fn with_jni<F>(callback: F) -> Self
    where F: Fn(&str, &Value) -> Result<Value> + Send + Sync + 'static {
        Self { jni_callback: Some(Box::new(callback)) }
    }

    fn call_jni(&self, sub: &str, args: &Value) -> Result<Value> {
        if let Some(cb) = &self.jni_callback { cb(sub, args) }
        else { Ok(json!({"error": "jni_callback not configured"})) }
    }
}

const HELP: &str = r#"android-device — model, OS version, battery, storage info.

Usage:
  android-device all          Full device info
  android-device info          Model, OS, build info
  android-device battery       Battery level and charging state
  android-device storage       Available storage
"#;

#[async_trait]
impl OffloadHandler for DeviceHandler {
    fn name(&self) -> &str { "device" }
    fn display_name(&self) -> &str { "android-device" }
    fn help(&self) -> &str { HELP }

    async fn execute(&self, args: Value) -> Result<OffloadResult> {
        let sub = args.get("subcommand")
            .and_then(|v| v.as_str())
            .unwrap_or("all");

        let result = self.call_jni(sub, &args)?;
        if result.get("error").is_some() {
            Ok(OffloadResult { success: false, data: result.clone(), error: result.get("error").and_then(|v| v.as_str()).map(|s| s.to_string()), exit_code: 1 })
        } else {
            Ok(OffloadResult::ok(result))
        }
    }

    fn check_permission(&self, _session_id: Option<&str>) -> PermissionState {
        PermissionState::Allowed
    }
}

impl Default for DeviceHandler { fn default() -> Self { Self::new() } }