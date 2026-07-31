// 음성 텍스트 정제 — 불필요한 문자/기호 제거
// 원본: VoiceTextSanitizer.kt

pub fn sanitize(text: &str) -> String {
    text.trim()
        .replace("  ", " ")
        .replace("[음악]", "")
        .replace("[음]", "")
        .replace("[박수]", "")
        .trim()
        .to_string()
}
