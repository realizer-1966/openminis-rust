// JNI 진입점 — Android 액티비티에서 호출되는 네이티브 메서드
// Kotlin의 nativeInit(), nativeShell(), nativeBashism() 등을 구현

#![cfg(target_os = "android")]

use std::os::raw::{c_char, jstring};
use std::ffi::{CStr, CString};
use jni::JNIEnv;
use jni::objects::JClass;

use crate::tool_dispatch::ToolDispatcher;
use crate::tools::file_read::FileReadTool;
use crate::tools::file_write::FileWriteTool;
use crate::tools::file_edit::FileEditTool;
use crate::tools::shell_execute::ShellExecuteTool;
use crate::tools::read_image::ReadImageTool;
use crate::tools::memory::{MemoryWriteTool, MemoryGetTool};
use crate::tools::browser_use::BrowserUseTool;

use minis_sandbox::shell::PersistentShell;
use minis_sandbox::BashismDetector;
use minis_minis_url::UrlResolver;

use std::sync::OnceLock;
use std::time::Duration;
use serde_json::json;

static DISPATCHER: OnceLock<std::sync::Arc<ToolDispatcher>> = OnceLock::new();
static BASHISM: OnceLock<BashismDetector> = OnceLock::new();

fn get_dispatcher() -> &'static std::sync::Arc<ToolDispatcher> {
    DISPATCHER.get_or_init(|| {
        let mut d = ToolDispatcher::new();
        d.register(Box::new(FileReadTool));
        d.register(Box::new(FileWriteTool));
        d.register(Box::new(FileEditTool));
        d.register(Box::new(ShellExecuteTool::default()));
        d.register(Box::new(ReadImageTool));
        d.register(Box::new(MemoryWriteTool));
        d.register(Box::new(MemoryGetTool));
        d.register(Box::new(BrowserUseTool));
        std::sync::Arc::new(d)
    })
}

fn get_bashism() -> &'static BashismDetector {
    BASHISM.get_or_init(BashismDetector::new)
}

fn jstring_from_string(env: &mut JNIEnv, s: &str) -> jstring {
    env.new_string(s).unwrap().into_raw()
}

fn string_from_jstring(env: &mut JNIEnv, s: jstring) -> String {
    env.get_string(s.into()).map(|jstr| jstr.to_string()).unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn Java_com_openminis_app_MainActivity_nativeInit(
    _env: JNIEnv,
    _class: JClass,
) -> bool {
    // 초기화 — 디스패처와 bashism 감지기 준비
    let _ = get_dispatcher();
    let _ = get_bashism();
    true
}

#[no_mangle]
pub extern "C" fn Java_com_openminis_app_MainActivity_nativeShell(
    mut env: JNIEnv,
    _class: JClass,
    command: jstring,
) -> jstring {
    let cmd = string_from_jstring(&mut env, command);
    let shell = PersistentShell::new();
    // 동기 실행 — JNI는 블로킹이 허용됨
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(shell.execute(&cmd, Duration::from_secs(30)));

    let json = match result {
        Ok(r) => json!({
            "exit_code": r.exit_code,
            "stdout": r.stdout,
            "stderr": r.stderr,
            "timed_out": r.timed_out,
            "new_urls": r.new_minis_urls,
        }).to_string(),
        Err(e) => json!({"error": e.to_string()}).to_string(),
    };

    jstring_from_string(&mut env, &json)
}

#[no_mangle]
pub extern "C" fn Java_com_openminis_app_MainActivity_nativeBashism(
    mut env: JNIEnv,
    _class: JClass,
    command: jstring,
) -> jstring {
    let cmd = string_from_jstring(&mut env, command);
    let detector = get_bashism();
    let detected = detector.detect(&cmd);
    let rules: Vec<_> = detected.iter().map(|r| json!({
        "name": r.name,
        "fix_hint": r.fix_hint,
    })).collect();

    let json = json!({
        "command": cmd,
        "has_bashisms": !rules.is_empty(),
        "rules": rules,
    }).to_string();

    jstring_from_string(&mut env, &json)
}

#[no_mangle]
pub extern "C" fn Java_com_openminis_app_MainActivity_nativeResolveUrl(
    mut env: JNIEnv,
    _class: JClass,
    url: jstring,
) -> jstring {
    let url_str = string_from_jstring(&mut env, url);
    let json = match UrlResolver::to_linux_path(&url_str) {
        Ok(p) => json!({"url": url_str, "linux_path": p.to_string_lossy()}).to_string(),
        Err(e) => json!({"error": e.to_string()}).to_string(),
    };
    jstring_from_string(&mut env, &json)
}

#[no_mangle]
pub extern "C" fn Java_com_openminis_app_MainActivity_nativeListTools(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    let dispatcher = get_dispatcher();
    let tools: Vec<_> = dispatcher.definitions().iter().map(|t| json!({
        "name": t.name,
        "description": t.description,
    })).collect();

    let json = json!({"tools": tools, "count": tools.len()}).to_string();
    jstring_from_string(&mut env, &json)
}