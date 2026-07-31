// 인자 파서 — CLI 스타일 인자를 JSON으로 변환
// 원본: OffloadArgs.kt

use serde_json::Value;
use std::collections::HashMap;

/// CLI 인자 파서 — --flag, --key value, --key=value, positional 지원
pub struct OffloadArgs {
    pub positional: Vec<String>,
    flags: HashMap<String, bool>,
    values: HashMap<String, String>,
}

impl OffloadArgs {
    pub fn from_argv(argv: &[String], boolean_flags: &[&str]) -> Self {
        let mut pos = Vec::new();
        let mut flags = HashMap::new();
        let mut values = HashMap::new();
        let bool_set: std::collections::HashSet<&str> = boolean_flags.iter().copied().collect();

        let mut i = 0;
        while i < argv.len() {
            let a = &argv[i];
            if a.starts_with("--") && a.contains('=') {
                let eq = a.find('=').unwrap();
                values.insert(a[2..eq].to_string(), a[eq + 1..].to_string());
            } else if a.starts_with("--") {
                let key = &a[2..];
                let next = argv.get(i + 1);
                if bool_set.contains(key) || next.is_none() || next.unwrap().starts_with('-') {
                    flags.insert(key.to_string(), true);
                } else {
                    values.insert(key.to_string(), next.unwrap().clone());
                    i += 1;
                }
            } else if a.starts_with('-') && a.len() > 1 {
                flags.insert(a[1..].to_string(), true);
            } else {
                pos.push(a.clone());
            }
            i += 1;
        }

        Self { positional: pos, flags, values }
    }

    pub fn has_flag(&self, name: &str) -> bool {
        self.flags.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(|s| s.as_str())
    }

    pub fn get_int(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(|s| s.parse().ok())
    }

    pub fn get_f64(&self, name: &str) -> Option<f64> {
        self.get(name).and_then(|s| s.parse().ok())
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.get(name).map(|s| matches!(s.to_lowercase().as_str(), "true" | "1" | "yes"))
    }

    /// JSON 객체로 변환 — OffloadHandler.execute()에 전달
    pub fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        if let Some(sub) = self.positional.first() {
            obj.insert("subcommand".into(), Value::String(sub.clone()));
        }
        for (k, v) in &self.values {
            obj.insert(k.clone(), Value::String(v.clone()));
        }
        for (k, _) in &self.flags {
            obj.insert(k.clone(), Value::Bool(true));
        }
        Value::Object(obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flags() {
        let args = OffloadArgs::from_argv(
            &["--today".into(), "list".into()],
            &["today", "all-day"],
        );
        assert!(args.has_flag("today"));
        assert_eq!(args.positional, vec!["list"]);
    }

    #[test]
    fn test_parse_key_value() {
        let args = OffloadArgs::from_argv(
            &["--title".into(), "Meeting".into(), "create".into()],
            &[],
        );
        assert_eq!(args.get("title"), Some("Meeting"));
        assert_eq!(args.positional, vec!["create"]);
    }

    #[test]
    fn test_parse_key_equals_value() {
        let args = OffloadArgs::from_argv(&["--max=10".into(), "list".into()], &[]);
        assert_eq!(args.get_int("max"), Some(10));
    }

    #[test]
    fn test_to_json() {
        let args = OffloadArgs::from_argv(
            &["create".into(), "--title".into(), "Test".into(), "--all-day".into()],
            &["all-day"],
        );
        let json = args.to_json();
        assert_eq!(json["subcommand"], "create");
        assert_eq!(json["title"], "Test");
        assert_eq!(json["all-day"], true);
    }
}