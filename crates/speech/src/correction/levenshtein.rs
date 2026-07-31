// Levenshtein 편집 거리 — 두 문자열 간의 최소 편집 연산 수
// 원본: PinyinNormalizer.kt의 levenshtein 함수
//
// 두 행 DP (공간 최적화). 문자 단위가 아닌 char 단위로 동작.

/// 두 문자열 간의 Levenshtein 편집 거리
pub fn levenshtein(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur: Vec<usize> = vec![0; b.len() + 1];

    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (cur[j - 1] + 1)
                .min(prev[j] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }

    prev[b.len()]
}

/// 정규화된 편집 유사도 (0.0 ~ 1.0)
pub fn similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    if a == b {
        return 1.0;
    }
    let dist = levenshtein(a, b);
    let max_len = a.chars().count().max(b.chars().count());
    1.0 - dist as f64 / max_len as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical() {
        assert_eq!(levenshtein("hello", "hello"), 0);
    }

    #[test]
    fn test_empty() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "abc"), 3);
    }

    #[test]
    fn test_single_edit() {
        assert_eq!(levenshtein("cat", "cot"), 1);
        assert_eq!(levenshtein("cat", "cats"), 1);
        assert_eq!(levenshtein("cats", "cat"), 1);
    }

    #[test]
    fn test_multiple_edits() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn test_unicode() {
        assert_eq!(levenshtein("张山", "张三"), 1);
        assert_eq!(levenshtein("안녕", "안녕"), 0);
    }

    #[test]
    fn test_similarity() {
        assert!((similarity("hello", "hello") - 1.0).abs() < 0.001);
        assert!((similarity("cat", "cot") - 0.6667).abs() < 0.01);
        assert!((similarity("", "") - 1.0).abs() < 0.001);
    }
}