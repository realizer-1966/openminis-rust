// 채팅 DAO — 세션/메시지 CRUD
// 원본: ChatDao.kt

use anyhow::Result;
use rusqlite::{params, Connection};
use chrono::{DateTime, Utc};
use crate::models::{ChatSessionEntity, MessageEntity};

pub fn insert_session(conn: &Connection, session: &ChatSessionEntity) -> Result<()> {
    conn.execute(
        "INSERT INTO chat_sessions (id, title, created_at, updated_at, provider_id, model_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            session.id, session.title,
            session.created_at.to_rfc3339(),
            session.updated_at.to_rfc3339(),
            session.provider_id, session.model_id,
        ],
    )?;
    Ok(())
}

pub fn insert_message(conn: &Connection, msg: &MessageEntity) -> Result<()> {
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            msg.id, msg.session_id, msg.role, msg.content,
            msg.created_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

pub fn list_messages(conn: &Connection, session_id: &str) -> Result<Vec<MessageEntity>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, role, content, created_at FROM messages WHERE session_id = ?1 ORDER BY created_at"
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        Ok(MessageEntity {
            id: row.get(0)?,
            session_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
}
