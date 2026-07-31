// Offload 통합 테스트 — 핸들러 등록, 인자 파싱, 권한 게이트

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OffloadRegistry, CalendarHandler, ContactsHandler, AlarmHandler,
                LocationHandler, WeatherHandler, ClipboardHandler, SpeakHandler};
    use crate::trait_def::*;
    use crate::args::OffloadArgs;
    use crate::gate::OffloadGate;
    use std::sync::Arc;

    #[test]
    fn test_registry_register_and_list() {
        let mut registry = OffloadRegistry::new();
        registry.register(Arc::new(CalendarHandler::new()));
        registry.register(Arc::new(ContactsHandler::new()));
        registry.register(Arc::new(AlarmHandler::new()));

        let names = registry.list_handlers();
        assert!(names.contains(&"calendar"));
        assert!(names.contains(&"contacts"));
        assert!(names.contains(&"alarm"));
        assert_eq!(names.len(), 3);
    }

    #[tokio::test]
    async fn test_calendar_no_jni_returns_data_with_error() {
        let handler = CalendarHandler::new();
        let args = serde_json::json!({"subcommand": "list"});
        let result = handler.execute(args).await.unwrap();
        // jni_callback 없으면 data에 error 표시, success=false
        assert!(!result.success);
        assert!(result.data.get("error").is_some());
    }

    #[tokio::test]
    async fn test_calendar_missing_title() {
        let handler = CalendarHandler::new();
        let args = serde_json::json!({"subcommand": "create", "start": "2024-01-01T10:00"});
        let result = handler.execute(args).await.unwrap();
        assert!(!result.success);
        assert_eq!(result.exit_code, 2);
        assert!(result.error.unwrap().contains("title"));
    }

    #[tokio::test]
    async fn test_calendar_missing_start() {
        let handler = CalendarHandler::new();
        let args = serde_json::json!({"subcommand": "create", "title": "Meeting"});
        let result = handler.execute(args).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("start"));
    }

    #[tokio::test]
    async fn test_calendar_unknown_subcommand() {
        let handler = CalendarHandler::new();
        let args = serde_json::json!({"subcommand": "frobnicate"});
        let result = handler.execute(args).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("unknown"));
    }

    #[tokio::test]
    async fn test_contacts_missing_query() {
        let handler = ContactsHandler::new();
        let args = serde_json::json!({"subcommand": "search"});
        let result = handler.execute(args).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("query"));
    }

    #[tokio::test]
    async fn test_alarm_missing_time() {
        let handler = AlarmHandler::new();
        let args = serde_json::json!({"subcommand": "set"});
        let result = handler.execute(args).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("time"));
    }

    #[tokio::test]
    async fn test_alarm_invalid_duration() {
        let handler = AlarmHandler::new();
        let args = serde_json::json!({"subcommand": "timer", "duration": "abc"});
        let result = handler.execute(args).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("duration"));
    }

    #[tokio::test]
    async fn test_location_missing_coords() {
        let handler = LocationHandler::new();
        let args = serde_json::json!({"subcommand": "geocode"});
        let result = handler.execute(args).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("lat"));
    }

    #[tokio::test]
    async fn test_location_lat_out_of_range() {
        let handler = LocationHandler::new();
        let args = serde_json::json!({"subcommand": "geocode", "lat": 999.0, "lon": 0.0});
        let result = handler.execute(args).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("range"));
    }

    #[tokio::test]
    async fn test_weather_missing_coords() {
        let handler = WeatherHandler::new();
        let args = serde_json::json!({"subcommand": "current"});
        let result = handler.execute(args).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("lat"));
    }

    #[tokio::test]
    async fn test_weather_lat_out_of_range() {
        let handler = WeatherHandler::new();
        let args = serde_json::json!({"subcommand": "current", "lat": 999.0, "lon": 0.0});
        let result = handler.execute(args).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("range"));
    }

    #[tokio::test]
    async fn test_clipboard_missing_text() {
        let handler = ClipboardHandler::new();
        let args = serde_json::json!({"subcommand": "set"});
        let result = handler.execute(args).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("text"));
    }

    #[tokio::test]
    async fn test_speak_missing_text() {
        let handler = SpeakHandler::new();
        let args = serde_json::json!({});
        let result = handler.execute(args).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("text"));
    }

    #[test]
    fn test_gate_allowed() {
        let handler = CalendarHandler::new();
        let result = OffloadGate::enforce(&handler, None);
        assert!(result.is_none()); // Allowed → None
    }

    #[test]
    fn test_offload_result_permission_denied() {
        let result = OffloadResult::permission_denied("android-calendar");
        assert_eq!(result.exit_code, 126);
        assert!(result.error.unwrap().contains("permission_denied"));
    }

    #[test]
    fn test_offload_args_parse() {
        let args = OffloadArgs::from_argv(
            &["--today".into(), "list".into()],
            &["today"],
        );
        assert!(args.has_flag("today"));
        assert_eq!(args.positional, vec!["list"]);
    }
}