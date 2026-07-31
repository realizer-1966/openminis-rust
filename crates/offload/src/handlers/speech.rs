// TTS / STT offload — 음성 합성 및 인식
// 원본: SpeakOffloadHandler.kt, SpeechOffloadHandler.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::trait_def::{OffloadHandler, OffloadResult, PermissionState};

pub struct SpeakHandler {
    jni_callback: Option<Box<dyn Fn(&str, &Value) -> Result<Value> + Send + Sync>>,
}

impl SpeakHandler {
    pub fn new() -> Self { Self { jni_callback: None } }
    pub fn with_jni<F>(callback: F) -> Self
    where F: Fn(&str, &Value) -> Result<Value> + Send + Sync + 'static {
        Self { jni_callback: Some(Box::new(callback)) }
    }
}

const HELP: &str = r#"android-speak — device TTS (text-to-speech).

Usage:
  android-speak <text> [--rate F] [--pitch F] [--volume F]
  android-speak --stop
  android-speak --status
"#;

#[async_trait]
impl OffloadHandler for SpeakHandler {
    fn name(&self) -> &str { "speak" }
    fn display_name(&self) -> &str { "android-speak" }
    fn help(&self) -> &str { HELP }

    async fn execute(&self, args: Value) -> Result<OffloadResult> {
        let text = args.get("text").and_then(|v| v.as_str());
        let stop = args.get("stop").and_then(|v| v.as_bool()).unwrap_or(false);
        let status = args.get("status").and_then(|v| v.as_bool()).unwrap_or(false);

        if !stop && !status && text.is_none() {
            return Ok(OffloadResult::bad_args("speak: text is required (or use --stop / --status)"));
        }

        if let Some(cb) = &self.jni_callback {
            let result = cb(if stop { "stop" } else if status { "status" } else { "speak" }, &args)?;
            if result.get("error").is_some() {
                Ok(OffloadResult { success: false, data: result.clone(), error: result.get("error").and_then(|v| v.as_str()).map(|s| s.to_string()), exit_code: 1 })
            } else {
                Ok(OffloadResult::ok(result))
            }
        } else {
            Ok(OffloadResult::error("jni_callback not configured", 1))
        }
    }

    fn check_permission(&self, _session_id: Option<&str>) -> PermissionState {
        PermissionState::Allowed
    }
}

impl Default for SpeakHandler { fn default() -> Self { Self::new() } }

// ── Speech (STT) ──

pub struct SpeechHandler {
    jni_callback: Option<Box<dyn Fn(&str, &Value) -> Result<Value> + Send + Sync>>,
}

impl SpeechHandler {
    pub fn new() -> Self { Self { jni_callback: None } }
    pub fn with_jni<F>(callback: F) -> Self
    where F: Fn(&str, &Value) -> Result<Value> + Send + Sync + 'static {
        Self { jni_callback: Some(Box::new(callback)) }
    }
}

const SPEECH_HELP: &str = r#"android-speech — microphone transcription (speech-to-text).

Usage:
  android-speech listen [--language BCP47] [--max N] [--timeout SEC]
  android-speech status
"#;

#[async_trait]
impl OffloadHandler for SpeechHandler {
    fn name(&self) -> &str { "speech" }
    fn display_name(&self) -> &str { "android-speech" }
    fn help(&self) -> &str { SPEECH_HELP }

    async fn execute(&self, args: Value) -> Result<OffloadResult> {
        let sub = args.get("subcommand")
            .and_then(|v| v.as_str())
            .unwrap_or("listen");

        if let Some(cb) = &self.jni_callback {
            let result = cb(sub, &args)?;
            if result.get("error").is_some() {
                Ok(OffloadResult { success: false, data: result.clone(), error: result.get("error").and_then(|v| v.as_str()).map(|s| s.to_string()), exit_code: 1 })
            } else {
                Ok(OffloadResult::ok(result))
            }
        } else {
            Ok(OffloadResult::error("jni_callback not configured", 1))
        }
    }

    fn check_permission(&self, _session_id: Option<&str>) -> PermissionState {
        PermissionState::Allowed
    }
}

impl Default for SpeechHandler { fn default() -> Self { Self::new() } }