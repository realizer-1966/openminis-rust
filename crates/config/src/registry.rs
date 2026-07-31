// 설정 레지스트리 — 모든 설정 필드를 중앙 관리

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    pub path: String,
    pub display_name: String,
    pub description: String,
    pub value: ConfigValue,
    pub default: ConfigValue,
    pub risk: ConfigRisk,
    pub revertable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ConfigValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigRisk {
    Normal,
    Sensitive,
    Dangerous,
}

pub struct ConfigRegistry {
    fields: HashMap<String, ConfigField>,
}

impl ConfigRegistry {
    pub fn new() -> Self {
        Self { fields: HashMap::new() }
    }

    pub fn register(&mut self, field: ConfigField) {
        self.fields.insert(field.path.clone(), field);
    }

    pub fn get(&self, path: &str) -> Option<&ConfigField> {
        self.fields.get(path)
    }

    pub fn set(&mut self, path: &str, value: ConfigValue) -> anyhow::Result<()> {
        let field = self.fields.get_mut(path)
            .ok_or_else(|| anyhow::anyhow!("Unknown config path: {}", path))?;
        field.value = value;
        Ok(())
    }

    pub fn list(&self) -> Vec<&ConfigField> {
        self.fields.values().collect()
    }
}

impl Default for ConfigRegistry {
    fn default() -> Self { Self::new() }
}
