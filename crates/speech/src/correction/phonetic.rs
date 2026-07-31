// 음소 정규화 — 텍스트를 음성 키로 변환하여 동음이의어/유사음 통합
// 원본: PhoneticNormalizer.kt (한자→병음, ICU Transliterator 사용)
//
// Rust에서는 ICU에 의존하지 않고 간단한 룩업 테이블로 구현.
// Android의 ICU 기반 병음과 정확히 동일한 키를 생성하려면
// `icu` crate의 transliterator가 필요하지만, 이 환경에서는
// 기본 라틴/숫자 패스스루 + 향후 확장 인터페이스만 제공.

use serde::{Deserialize, Serialize};

/// 음소 정규화 trait — 언어별 구현
pub trait PhoneticNormalizer: Send + Sync {
    /// 텍스트를 음성 키로 변환. 빈 문자열이면 음성 내용 없음.
    fn normalize(&self, text: &str) -> String;

    /// 두 문자열의 음성 유사도 (0.0 ~ 1.0)
    fn similarity(&self, a: &str, b: &str) -> f64 {
        if a.is_empty() && b.is_empty() {
            return 1.0;
        }
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }
        if a == b {
            return 1.0;
        }
        let distance = crate::correction::levenshtein::levenshtein(a, b);
        let max_len = a.chars().count().max(b.chars().count());
        1.0 - distance as f64 / max_len as f64
    }
}

/// 라틴 패스스루 정규화기 — 영문/숫자를 소문자로 정규화
pub struct LatinNormalizer;

impl PhoneticNormalizer for LatinNormalizer {
    fn normalize(&self, text: &str) -> String {
        text.trim().to_lowercase()
    }
}

/// 레지스트리 — 언어 코드로 정규화기 선택
pub fn normalizer_for_locale(locale: &str) -> Option<Box<dyn PhoneticNormalizer>> {
    let base = locale.to_lowercase();
    let base = base.split(['-', '_']).next().unwrap_or(&base);
    match base {
        "en" | "" => Some(Box::new(LatinNormalizer)),
        // zh: 향후 ICU 기반 병음 정규화기 추가
        // ja: 향후 가나 정규화기 추가
        _ => Some(Box::new(LatinNormalizer)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latin_normalize() {
        let n = LatinNormalizer;
        assert_eq!(n.normalize("Hello World"), "hello world");
        assert_eq!(n.normalize(""), "");
    }

    #[test]
    fn test_similarity_identical() {
        let n = LatinNormalizer;
        assert!((n.similarity("hello", "hello") - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_similarity_different() {
        let n = LatinNormalizer;
        let sim = n.similarity("hello", "hallo");
        assert!(sim > 0.7 && sim < 1.0, "similarity should be high: {}", sim);
    }

    #[test]
    fn test_similarity_empty() {
        let n = LatinNormalizer;
        assert!((n.similarity("", "") - 1.0).abs() < 0.001);
        assert!((n.similarity("a", "") - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_normalizer_for_locale() {
        assert!(normalizer_for_locale("en").is_some());
        assert!(normalizer_for_locale("en-US").is_some());
        assert!(normalizer_for_locale("zh").is_some());
        assert!(normalizer_for_locale("").is_some());
    }
}