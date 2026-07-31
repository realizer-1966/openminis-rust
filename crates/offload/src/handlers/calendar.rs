// 캘린더 offload — 일정 조회/생성/수정/삭제
// 원본: CalendarOffloadHandler.kt
// JNI를 통해 Android CalendarContract ContentProvider 호출

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::trait_def::{OffloadHandler, OffloadResult, PermissionState};

pub struct CalendarHandler {
    /// JNI 콜백: 실제 Android API 호출을 Kotlin에 위임
    /// (subcommand, args_json) -> result_json
    jni_callback: Option<Box<dyn Fn(&str, &Value) -> Result<Value> + Send + Sync>>,
}

impl CalendarHandler {
    pub fn new() -> Self {
        Self { jni_callback: None }
    }

    pub fn with_jni<F>(callback: F) -> Self
    where
        F: Fn(&str, &Value) -> Result<Value> + Send + Sync + 'static,
    {
        Self { jni_callback: Some(Box::new(callback)) }
    }

    fn call_jni(&self, sub: &str, args: &Value) -> Result<Value> {
        if let Some(cb) = &self.jni_callback {
            cb(sub, args)
        } else {
            Ok(json!({"error": "jni_callback not configured"}))
        }
    }
}

const HELP: &str = r#"android-calendar — list, create, update, delete events; query free/busy.

Usage:
  android-calendar list [--today | --days N | --start S --end E] [--limit N] [--calendar NAME]
  android-calendar create --title T --start S [--end E] [--notes N] [--location L] [--all-day]
  android-calendar update --id <event_id> [--title ...] [--start ...] [--end ...]
  android-calendar delete --id <event_id>
  android-calendar freebusy --start <ISO> --end <ISO>
  android-calendar calendars                List writable calendars
"#;

#[async_trait]
impl OffloadHandler for CalendarHandler {
    fn name(&self) -> &str { "calendar" }
    fn display_name(&self) -> &str { "android-calendar" }

    fn help(&self) -> &str { HELP }

    async fn execute(&self, args: Value) -> Result<OffloadResult> {
        let sub = args.get("subcommand")
            .and_then(|v| v.as_str())
            .unwrap_or("list");

        // 인자 검증
        match sub {
            "list" | "calendars" | "freebusy" => {}
            "create" => {
                if args.get("title").and_then(|v| v.as_str()).is_none() {
                    return Ok(OffloadResult::bad_args("create: --title is required"));
                }
                if args.get("start").and_then(|v| v.as_str()).is_none() {
                    return Ok(OffloadResult::bad_args("create: --start is required"));
                }
            }
            "update" | "delete" => {
                if args.get("id").and_then(|v| v.as_str()).is_none() {
                    return Ok(OffloadResult::bad_args(&format!("{}: --id is required", sub)));
                }
            }
            _ => {
                return Ok(OffloadResult::bad_args(&format!("unknown subcommand '{}'", sub)));
            }
        }

        // JNI 호출
        let result = self.call_jni(sub, &args)?;
        if result.get("error").is_some() {
            Ok(OffloadResult {
                success: false,
                data: result.clone(),
                error: result.get("error").and_then(|v| v.as_str()).map(|s| s.to_string()),
                exit_code: 1,
            })
        } else {
            Ok(OffloadResult::ok(result))
        }
    }

    fn check_permission(&self, _session_id: Option<&str>) -> PermissionState {
        PermissionState::Allowed
    }
}

impl Default for CalendarHandler {
    fn default() -> Self { Self::new() }
}