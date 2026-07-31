// 프로바이더 DAO — 설정 CRUD
// 원본: ProviderConfigDao.kt

use anyhow::Result;
use rusqlite::{params, Connection};
use chrono::{DateTime, Utc};
use crate::models::ProviderInstanceEntity;

pub fn insert_provider(conn: &Connection, p: &ProviderInstanceEntity) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO provider_configs (id, provider_type, api_key, base_url, config_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![p.id, p.provider_type, p.api_key, p.base_url, p.config_json, p.created_at.to_rfc3339()],
    )?;
    Ok(())
}

pub fn list_providers(conn: &Connection) -> Result<Vec<ProviderInstanceEntity>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider_type, api_key, base_url, config_json, created_at FROM provider_configs ORDER BY created_at"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ProviderInstanceEntity {
            id: row.get(0)?,
            provider_type: row.get(1)?,
            api_key: row.get(2)?,
            base_url: row.get(3)?,
            config_json: row.get(4)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
}
