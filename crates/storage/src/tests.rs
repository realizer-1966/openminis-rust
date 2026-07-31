// 스토리지 테스트 — SQLite DB 세션/메시지/프로바이더 CRUD

#[cfg(test)]
mod tests {
    use crate::db::AppDatabase;
    use crate::models::{ChatSessionEntity, MessageEntity, ProviderInstanceEntity};
    use crate::chat_dao;
    use crate::provider_dao;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_db() -> AppDatabase {
        AppDatabase::in_memory().unwrap()
    }

    fn make_session() -> ChatSessionEntity {
        let now = Utc::now();
        ChatSessionEntity {
            id: Uuid::new_v4().to_string(),
            title: "Test Chat".into(),
            created_at: now,
            updated_at: now,
            provider_id: Some("anthropic".into()),
            model_id: Some("claude-sonnet-4-5".into()),
        }
    }

    fn make_message(session_id: &str, role: &str, content: &str) -> MessageEntity {
        MessageEntity {
            id: Uuid::new_v4().to_string(),
            session_id: session_id.into(),
            role: role.into(),
            content: content.into(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_create_db() {
        let db = create_test_db();
        let result = db.with_conn(|c| -> anyhow::Result<i64> {
            Ok(c.query_row("SELECT 1", [], |row| row.get::<_, i64>(0))?)
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_insert_and_list_session() {
        let db = create_test_db();
        let session = make_session();
        db.with_conn(|c| chat_dao::insert_session(c, &session)).unwrap();

        let sessions = db.with_conn(|c| -> anyhow::Result<Vec<(String, String)>> {
            let mut stmt = c.prepare("SELECT id, title FROM chat_sessions")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
        }).unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].0, session.id);
        assert_eq!(sessions[0].1, "Test Chat");
    }

    #[test]
    fn test_insert_and_list_messages() {
        let db = create_test_db();
        let session = make_session();
        db.with_conn(|c| chat_dao::insert_session(c, &session)).unwrap();

        let msg1 = make_message(&session.id, "user", "Hello");
        let msg2 = make_message(&session.id, "assistant", "Hi there!");
        db.with_conn(|c| chat_dao::insert_message(c, &msg1)).unwrap();
        db.with_conn(|c| chat_dao::insert_message(c, &msg2)).unwrap();

        let messages = db.with_conn(|c| chat_dao::list_messages(c, &session.id)).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[1].role, "assistant");
    }

    #[test]
    fn test_messages_ordered_by_time() {
        let db = create_test_db();
        let session = make_session();
        db.with_conn(|c| chat_dao::insert_session(c, &session)).unwrap();

        let early = make_message(&session.id, "user", "first");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let late = make_message(&session.id, "user", "second");
        db.with_conn(|c| chat_dao::insert_message(c, &late)).unwrap();
        db.with_conn(|c| chat_dao::insert_message(c, &early)).unwrap();

        let messages = db.with_conn(|c| chat_dao::list_messages(c, &session.id)).unwrap();
        assert_eq!(messages.len(), 2);
        // 시간순 정렬 — early가 먼저
        assert_eq!(messages[0].content, "first");
        assert_eq!(messages[1].content, "second");
    }

    #[test]
    fn test_provider_crud() {
        let db = create_test_db();
        let provider = ProviderInstanceEntity {
            id: Uuid::new_v4().to_string(),
            provider_type: "anthropic".into(),
            api_key: Some("sk-test".into()),
            base_url: None,
            config_json: None,
            created_at: Utc::now(),
        };
        db.with_conn(|c| provider_dao::insert_provider(c, &provider)).unwrap();

        let providers = db.with_conn(|c| provider_dao::list_providers(c)).unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].provider_type, "anthropic");
    }

    #[test]
    fn test_multiple_sessions() {
        let db = create_test_db();
        let s1 = make_session();
        let s2 = make_session();
        db.with_conn(|c| chat_dao::insert_session(c, &s1)).unwrap();
        db.with_conn(|c| chat_dao::insert_session(c, &s2)).unwrap();

        let m1 = make_message(&s1.id, "user", "msg in s1");
        let m2 = make_message(&s2.id, "user", "msg in s2");
        db.with_conn(|c| chat_dao::insert_message(c, &m1)).unwrap();
        db.with_conn(|c| chat_dao::insert_message(c, &m2)).unwrap();

        let s1_msgs = db.with_conn(|c| chat_dao::list_messages(c, &s1.id)).unwrap();
        let s2_msgs = db.with_conn(|c| chat_dao::list_messages(c, &s2.id)).unwrap();
        assert_eq!(s1_msgs.len(), 1);
        assert_eq!(s2_msgs.len(), 1);
        assert_eq!(s1_msgs[0].content, "msg in s1");
        assert_eq!(s2_msgs[0].content, "msg in s2");
    }

    #[test]
    fn test_session_isolation() {
        let db = create_test_db();
        let s1 = make_session();
        let s2 = make_session();
        db.with_conn(|c| chat_dao::insert_session(c, &s1)).unwrap();
        db.with_conn(|c| chat_dao::insert_session(c, &s2)).unwrap();

        let m1 = make_message(&s1.id, "user", "private to s1");
        let m2 = make_message(&s2.id, "user", "private to s2");
        db.with_conn(|c| chat_dao::insert_message(c, &m1)).unwrap();
        db.with_conn(|c| chat_dao::insert_message(c, &m2)).unwrap();

        // 세션 1의 메시지에 세션 2의 메시지가 없어야 함
        let s1_msgs = db.with_conn(|c| chat_dao::list_messages(c, &s1.id)).unwrap();
        assert!(s1_msgs.iter().all(|m| m.session_id == s1.id));
    }
}