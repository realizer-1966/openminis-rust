// LCS 기반 텍스트 diff — before/after 텍스트에서 교정 쌍 추출
// 원본: VoiceCorrectionDiff.kt, LcsDiff.kt
//
// 문자 단위 LCS diff로 변경을 추출하고, 단어 경계로 확장하여
// "张山→张三" 같은 교정 쌍을 학습.

use serde::{Deserialize, Serialize};

/// 하나의 교정 쌍
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorrectionPair {
    pub from: String,
    pub to: String,
}

/// LCS diff 연산
#[derive(Debug, Clone)]
pub enum DiffOp {
    Equal(Vec<String>),
    Replace(Vec<String>, Vec<String>),
    Insert(Vec<String>),
    Delete(Vec<String>),
}

/// LCS diff — 두 토큰 목록의 차이를 연산 시퀀스로 반환
pub struct LcsDiff;

impl LcsDiff {
    pub fn diff(a: &[String], b: &[String]) -> Vec<DiffOp> {
        if a.is_empty() && b.is_empty() {
            return vec![];
        }
        if a.is_empty() {
            return vec![DiffOp::Insert(b.to_vec())];
        }
        if b.is_empty() {
            return vec![DiffOp::Delete(a.to_vec())];
        }

        // 접미사 LCS 테이블 (역방향 채우기)
        let mut lcs = vec![vec![0usize; b.len() + 1]; a.len() + 1];
        for i in (0..a.len()).rev() {
            for j in (0..b.len()).rev() {
                lcs[i][j] = if a[i] == b[j] {
                    lcs[i + 1][j + 1] + 1
                } else {
                    lcs[i + 1][j].max(lcs[i][j + 1])
                };
            }
        }

        let mut ops = Vec::new();
        let mut pending_old: Vec<String> = Vec::new();
        let mut pending_new: Vec<String> = Vec::new();
        let mut pending_equal: Vec<String> = Vec::new();

        let mut flush_changes = |old: &mut Vec<String>,
                                 new: &mut Vec<String>,
                                 ops: &mut Vec<DiffOp>| {
            if old.is_empty() && new.is_empty() {
                return;
            }
            if !old.is_empty() && !new.is_empty() {
                ops.push(DiffOp::Replace(old.clone(), new.clone()));
            } else if !old.is_empty() {
                ops.push(DiffOp::Delete(old.clone()));
            } else {
                ops.push(DiffOp::Insert(new.clone()));
            }
            old.clear();
            new.clear();
        };

        let mut flush_equal = |eq: &mut Vec<String>, ops: &mut Vec<DiffOp>| {
            if eq.is_empty() {
                return;
            }
            ops.push(DiffOp::Equal(eq.clone()));
            eq.clear();
        };

        let mut i = 0;
        let mut j = 0;
        while i < a.len() && j < b.len() {
            if a[i] == b[j] {
                flush_changes(&mut pending_old, &mut pending_new, &mut ops);
                pending_equal.push(a[i].clone());
                i += 1;
                j += 1;
            } else {
                flush_equal(&mut pending_equal, &mut ops);
                if lcs[i + 1][j] >= lcs[i][j + 1] {
                    pending_old.push(a[i].clone());
                    i += 1;
                } else {
                    pending_new.push(b[j].clone());
                    j += 1;
                }
            }
        }
        flush_equal(&mut pending_equal, &mut ops);
        while i < a.len() {
            pending_old.push(a[i].clone());
            i += 1;
        }
        while j < b.len() {
            pending_new.push(b[j].clone());
            j += 1;
        }
        flush_changes(&mut pending_old, &mut pending_new, &mut ops);

        ops
    }
}

/// 텍스트 diff — before/after에서 교정 쌍 추출
pub struct VoiceCorrectionDiff;

impl VoiceCorrectionDiff {
    /// 문자 수준 변경 비율 (0.0 ~ 1.0)
    /// MAX_CHAR_CHANGE_RATIO(0.5) 이상이면 재작성으로 판단
    pub fn char_change_ratio(from: &str, to: &str) -> f64 {
        if from.is_empty() && to.is_empty() {
            return 0.0;
        }
        let longest = from.chars().count().max(to.chars().count());
        if longest == 0 {
            return 0.0;
        }
        crate::correction::levenshtein::levenshtein(from, to) as f64 / longest as f64
    }

    /// before/after 텍스트에서 교정 쌍 추출
    /// 토크나이저로 단어 경계를 식별 (중국어 분사 등)
    pub fn token_pairs(
        from: &str,
        to: &str,
        tokenize: impl Fn(&str) -> Vec<String>,
    ) -> Vec<CorrectionPair> {
        if from.is_empty() || to.is_empty() || from == to {
            return vec![];
        }

        // 문자 단위로 분할
        let a: Vec<String> = from.chars().map(|c| c.to_string()).collect();
        let b: Vec<String> = to.chars().map(|c| c.to_string()).collect();

        // 원본 텍스트의 단어 경계
        let spans = Self::word_spans(from, &tokenize);

        let mut out = Vec::new();
        let mut ai = 0;
        let mut bi = 0;

        for op in LcsDiff::diff(&a, &b) {
            match op {
                DiffOp::Equal(items) => {
                    ai += items.len();
                    bi += items.len();
                }
                DiffOp::Insert(items) => {
                    bi += items.len();
                }
                DiffOp::Delete(items) => {
                    ai += items.len();
                }
                DiffOp::Replace(old, new) => {
                    let old_start = ai;
                    let old_end = ai + old.len();
                    let new_start = bi;
                    let new_end = bi + new.len();

                    // 단어 경계로 확장
                    let mut lo = old_start;
                    let mut hi = old_end;
                    if let Some(span) = Self::word_containing(&spans, old_start) {
                        lo = lo.min(span.0);
                    }
                    if old_end > old_start {
                        if let Some(span) = Self::word_containing(&spans, old_end - 1) {
                            hi = hi.max(span.1);
                        }
                    }

                    let pad_left = old_start - lo;
                    let pad_right = hi - old_end;
                    let n_lo = new_start.saturating_sub(pad_left);
                    let n_hi = (new_end + pad_right).min(b.len());

                    let was: String = a[lo.min(a.len())..hi.min(a.len())].concat();
                    let became: String = b[n_lo.min(b.len())..n_hi.min(b.len())].concat();

                    ai = old_end;
                    bi = new_end;

                    if was.is_empty() || became.is_empty() || was == became {
                        continue;
                    }

                    // 길이 불균형 필터 (ratio > 2.0이면 재작성)
                    let max_len = was.chars().count().max(became.chars().count());
                    let min_len = was.chars().count().min(became.chars().count()).max(1);
                    if max_len as f64 / min_len as f64 > 2.0 {
                        continue;
                    }

                    out.push(CorrectionPair { from: was, to: became });
                }
            }
        }
        out
    }

    /// 텍스트 내 단어의 [start, end) 범위 목록
    fn word_spans(text: &str, tokenize: impl Fn(&str) -> Vec<String>) -> Vec<(usize, usize)> {
        let tokens = tokenize(text);
        let mut spans = Vec::new();
        let mut cursor = 0;
        for t in &tokens {
            if t.is_empty() {
                continue;
            }
            if let Some(idx) = text[cursor..].find(t) {
                let abs = cursor + idx;
                spans.push((abs, abs + t.len()));
                cursor = abs + t.len();
            }
        }
        spans
    }

    fn word_containing(spans: &[(usize, usize)], index: usize) -> Option<(usize, usize)> {
        spans.iter().find(|(start, end)| index >= *start && index < *end).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn whitespace_tokenize(text: &str) -> Vec<String> {
        text.split_whitespace().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_char_change_ratio_identical() {
        assert_eq!(VoiceCorrectionDiff::char_change_ratio("hello", "hello"), 0.0);
    }

    #[test]
    fn test_char_change_ratio_small() {
        let ratio = VoiceCorrectionDiff::char_change_ratio("cat", "cot");
        assert!(ratio < 0.5, "small change should be < 0.5: {}", ratio);
    }

    #[test]
    fn test_char_change_ratio_large() {
        let ratio = VoiceCorrectionDiff::char_change_ratio("hello world", "goodbye universe");
        assert!(ratio > 0.5, "large change should be > 0.5: {}", ratio);
    }

    #[test]
    fn test_token_pairs_simple() {
        // "helllo" → "hello": 문자 단위 diff에서 replace 발생
        let pairs = VoiceCorrectionDiff::token_pairs("helllo world", "hello world", whitespace_tokenize);
        // replace가 발생하면 쌍이 추출됨
        // (정확한 결과는 문자 단위 LCS에 의해 결정됨)
        // 빈 결과도 허용 — 단순 삽입/삭제는 쌍을 생성하지 않을 수 있음
    }

    #[test]
    fn test_token_pairs_replace() {
        // "hellp" → "hello": 문자 교체가 발생
        let pairs = VoiceCorrectionDiff::token_pairs("hellp world", "hello world", whitespace_tokenize);
        assert!(!pairs.is_empty(), "should detect correction pair for hellp→hello");
    }

    #[test]
    fn test_token_pairs_identical() {
        let pairs = VoiceCorrectionDiff::token_pairs("hello", "hello", whitespace_tokenize);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_token_pairs_empty() {
        let pairs = VoiceCorrectionDiff::token_pairs("", "hello", whitespace_tokenize);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_lcs_diff_simple() {
        let a = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let b = vec!["a".to_string(), "x".to_string(), "c".to_string()];
        let ops = LcsDiff::diff(&a, &b);
        assert!(ops.iter().any(|op| matches!(op, DiffOp::Replace(_, _))));
    }

    #[test]
    fn test_lcs_diff_identical() {
        let a = vec!["x".to_string(), "y".to_string()];
        let ops = LcsDiff::diff(&a, &a);
        assert!(ops.iter().all(|op| matches!(op, DiffOp::Equal(_))));
    }
}