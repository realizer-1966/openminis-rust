// 연락처 offload — 조회/검색/상세/삭제
// 원본: ContactsOffloadHandler.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::trait_def::{OffloadHandler, OffloadResult, PermissionState};

pub struct ContactsHandler {
    jni_callback: Option<Box<dyn Fn(&str, &Value) -> Result<Value> + Send + Sync>>,
}

impl ContactsHandler {
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

const HELP: &str = r#"android-contacts — list, search, get, delete contacts.

Usage:
  android-contacts list [--max N]
  android-contacts search <query> [--max N]
  android-contacts get <id>
  android-contacts delete <id>
"#;

#[async_trait]
impl OffloadHandler for ContactsHandler {
    fn name(&self) -> &str { "contacts" }
    fn display_name(&self) -> &str { "android-contacts" }
    fn help(&self) -> &str { HELP }

    async fn execute(&self, args: Value) -> Result<OffloadResult> {
        let sub = args.get("subcommand")
            .and_then(|v| v.as_str())
            .unwrap_or("list");

        match sub {
            "list" => {}
            "search" => {
                if args.get("query").and_then(|v| v.as_str()).is_none() {
                    return Ok(OffloadResult::bad_args("search: query is required"));
                }
            }
            "get" | "delete" => {
                if args.get("id").and_then(|v| v.as_str()).is_none() {
                    return Ok(OffloadResult::bad_args(&format!("{}: id is required", sub)));
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

impl Default for ContactsHandler { fn default() -> Self { Self::new() } }