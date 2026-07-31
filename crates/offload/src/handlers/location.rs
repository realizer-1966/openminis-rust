// 위치 offload — 현재 위치, 역지오코딩, 순방향 지오코딩
// 원본: LocationOffloadHandler.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::trait_def::{OffloadHandler, OffloadResult, PermissionState};

pub struct LocationHandler {
    jni_callback: Option<Box<dyn Fn(&str, &Value) -> Result<Value> + Send + Sync>>,
}

impl LocationHandler {
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

const HELP: &str = r#"android-location — current location, reverse geocoding, forward geocoding.

Usage:
  android-location current [--timeout SEC]
  android-location geocode --lat <lat> --lon <lon>
  android-location forward --address "<addr>"
"#;

#[async_trait]
impl OffloadHandler for LocationHandler {
    fn name(&self) -> &str { "location" }
    fn display_name(&self) -> &str { "android-location" }
    fn help(&self) -> &str { HELP }

    async fn execute(&self, args: Value) -> Result<OffloadResult> {
        let sub = args.get("subcommand")
            .and_then(|v| v.as_str())
            .unwrap_or("current");

        match sub {
            "current" => {}
            "geocode" => {
                let lat = args.get("lat").and_then(|v| v.as_f64());
                let lon = args.get("lon").or_else(|| args.get("lng")).and_then(|v| v.as_f64());
                if lat.is_none() || lon.is_none() {
                    return Ok(OffloadResult::bad_args("geocode: --lat and --lon are required"));
                }
                let lat = lat.unwrap();
                if !(-90.0..=90.0).contains(&lat) {
                    return Ok(OffloadResult::bad_args("latitude out of range (-90..90)"));
                }
            }
            "forward" => {
                if args.get("address").and_then(|v| v.as_str()).is_none() {
                    return Ok(OffloadResult::bad_args("forward: --address is required"));
                }
            }
            _ => return Ok(OffloadResult::bad_args(&format!("unknown subcommand '{}'", sub))),
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

impl Default for LocationHandler { fn default() -> Self { Self::new() } }