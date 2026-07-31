// 알람 offload — 알람 설정, 타이머, 조회, 취소
// 원본: AlarmOffloadHandler.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::trait_def::{OffloadHandler, OffloadResult, PermissionState};

pub struct AlarmHandler {
    jni_callback: Option<Box<dyn Fn(&str, &Value) -> Result<Value> + Send + Sync>>,
}

impl AlarmHandler {
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

const HELP: &str = r#"android-alarm — schedule alarms and timers, list, or cancel them.

Usage:
  android-alarm set --time HH:MM|ISO --label L [--repeat ONCE|DAILY|WEEKDAYS]
  android-alarm timer --duration <seconds|5m|1h> [--label L]
  android-alarm list
  android-alarm cancel --id <alarm_id>
  android-alarm cancel --all
  android-alarm open                     Open the system Clock app
"#;

fn parse_duration(s: &str) -> Option<u64> {
    if s.is_empty() { return None; }
    if let Ok(secs) = s.parse::<u64>() { return Some(secs); }
    let lower = s.to_lowercase();
    if let Some(n) = lower.strip_suffix('s') { return n.parse().ok(); }
    if let Some(n) = lower.strip_suffix('m') { return n.parse::<u64>().ok().map(|m| m * 60); }
    if let Some(n) = lower.strip_suffix('h') { return n.parse::<u64>().ok().map(|h| h * 3600); }
    None
}

#[async_trait]
impl OffloadHandler for AlarmHandler {
    fn name(&self) -> &str { "alarm" }
    fn display_name(&self) -> &str { "android-alarm" }
    fn help(&self) -> &str { HELP }

    async fn execute(&self, args: Value) -> Result<OffloadResult> {
        let sub = args.get("subcommand")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match sub {
            "set" | "schedule" => {
                let time = args.get("time").and_then(|v| v.as_str());
                if time.is_none() {
                    return Ok(OffloadResult::bad_args("set: --time is required (HH:MM or ISO)"));
                }
            }
            "timer" => {
                let duration = args.get("duration").and_then(|v| v.as_str());
                if duration.is_none() {
                    return Ok(OffloadResult::bad_args("timer: --duration is required"));
                }
                if let Some(dur) = duration {
                    if parse_duration(dur).is_none() {
                        return Ok(OffloadResult::bad_args(&format!("invalid duration '{}', use seconds, 5m, 1h", dur)));
                    }
                }
            }
            "open" | "list" | "cancel" => {}
            "" => return Ok(OffloadResult::bad_args("no subcommand specified")),
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

impl Default for AlarmHandler { fn default() -> Self { Self::new() } }