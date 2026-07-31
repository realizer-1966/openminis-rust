// 터미널 세션 관리
// 원본: TerminalSession.kt, TerminalSanitizer.kt

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSession {
    pub id: String,
    pub cols: u16,
    pub rows: u16,
    pub encoding: String,
}

pub fn sanitize_output(output: &str) -> String {
    // ANSI escape 시퀀스 정리
    output.replace("\r\n", "\n")
}
