// 사진 offload — 기기 사진 라이브러리 조회
// 원본: PhotosOffloadHandler.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::trait_def::{OffloadHandler, OffloadResult, PermissionState};

pub struct PhotosHandler {
    jni_callback: Option<Box<dyn Fn(&str, &Value) -> Result<Value> + Send + Sync>>,
}

impl PhotosHandler {
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

const HELP: &str = r#"android-photos — query the device photo library via MediaStore.

Usage:
  android-photos list [--max N]
  android-photos stats
  android-photos near --lat L --lon L [--radius KM] [--max N]
"#;

#[async_trait]
impl OffloadHandler for PhotosHandler {
    fn name(&self) -> &str { "photos" }
    fn display_name(&self) -> &str { "android-photos" }
    fn help(&self) -> &str { HELP }

    async fn execute(&self, args: Value) -> Result<OffloadResult> {
        let sub = args.get("subcommand")
            .and_then(|v| v.as_str())
            .unwrap_or("list");

        if sub == "near" {
            if args.get("lat").and_then(|v| v.as_f64()).is_none()
                || args.get("lon").and_then(|v| v.as_f64()).is_none() {
                return Ok(OffloadResult::bad_args("near: --lat and --lon are required"));
            }
        }

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

impl Default for PhotosHandler { fn default() -> Self { Self::new() } }