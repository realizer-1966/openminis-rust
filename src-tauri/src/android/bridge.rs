// Tauri Android 플러그인 — Rust 코어와 Kotlin의 FFI 브릿지
// src-tauri/src/android/bridge.rs
//
// JNI를 통해 Kotlin OffloadService를 호출하는 Rust 쪽 구현

use anyhow::Result;
use serde_json::Value;

/// JNI 콜백 trait — OffloadHandler가 Kotlin 메서드를 호출할 때 사용
pub trait JniCallback: Send + Sync {
    /// (handler_name, subcommand, args_json) → result_json
    fn call(&self, handler: &str, subcommand: &str, args: &Value) -> Result<Value>;
}

/// JNI를 통해 Kotlin OffloadService.execute() 호출
/// 실제 구현은 jni crate을 사용하여 JNIEnv 획득 후 메서드 호출
pub struct JniOffloadDispatcher {
    // 실제 빌드에서는 JavaVM, OffloadService JObject를 보관
    // _vm: jni::JavaVM,
    // _service: jni::objects::GlobalRef,
}

impl JniOffloadDispatcher {
    pub fn new() -> Self {
        Self {}
    }
}

impl JniCallback for JniOffloadDispatcher {
    fn call(&self, _handler: &str, _subcommand: &str, _args: &Value) -> Result<Value> {
        // TODO: 실제 JNI 호출 구현
        // 1. JNIEnv 획득 (vm.attach_current_thread())
        // 2. OffloadService 클래스 찾기
        // 3. 메서드 ID 획득 (handler_name + subcommand로 분기)
        // 4. JSON 문자열 인자 전달
        // 5. 결과 JSON 문자열 수신 → Value로 파싱
        Ok(serde_json::json!({"error": "jni not implemented in this build"}))
    }
}

impl Default for JniOffloadDispatcher {
    fn default() -> Self { Self::new() }
}