// 문장 분할 — 음성 인식 결과를 문장 단위로 분할
// 원본: SpeechSentenceSplitter.kt

pub fn split_sentences(text: &str) -> Vec<String> {
    // 한국어, 일본어, 중국어 문장 구분자도 처리
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if ".!?。！？\n".contains(ch) {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }

    let remaining = current.trim().to_string();
    if !remaining.is_empty() {
        sentences.push(remaining);
    }

    sentences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split() {
        let result = split_sentences("Hello world. This is a test! Right?");
        assert_eq!(result.len(), 3);
    }
}
