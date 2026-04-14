use chrono::{TimeZone, Utc};
use cxusage::parser::parse_usage_snapshot;

#[test]
fn parses_usage_snapshot_from_token_count_event() {
    let line = r#"{"timestamp":"2026-04-14T08:22:00.000Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":1000,"cached_input_tokens":900,"output_tokens":100,"reasoning_output_tokens":10,"total_tokens":1110},"last_token_usage":{"input_tokens":10,"cached_input_tokens":9,"output_tokens":1,"reasoning_output_tokens":0,"total_tokens":11},"model_context_window":258400},"rate_limits":{"limit_id":"codex","limit_name":null,"primary":{"used_percent":8.0,"window_minutes":300,"resets_at":1774351986},"secondary":{"used_percent":56.0,"window_minutes":10080,"resets_at":1774457050},"credits":null,"plan_type":"pro"}}}"#;

    let snapshot = parse_usage_snapshot(line, "session-123").expect("snapshot should parse");

    assert_eq!(snapshot.session_id, "session-123");
    assert_eq!(
        snapshot.observed_at,
        Utc.with_ymd_and_hms(2026, 4, 14, 8, 22, 0).unwrap()
    );
    assert_eq!(snapshot.primary.used_percent, Some(8.0));
    assert_eq!(snapshot.primary.window_minutes, Some(300));
    assert_eq!(
        snapshot.primary.resets_at,
        Some(Utc.timestamp_opt(1_774_351_986, 0).unwrap())
    );
    assert_eq!(snapshot.secondary.used_percent, Some(56.0));
    assert_eq!(snapshot.secondary.window_minutes, Some(10_080));
    assert_eq!(
        snapshot.secondary.resets_at,
        Some(Utc.timestamp_opt(1_774_457_050, 0).unwrap())
    );
    assert_eq!(snapshot.plan_type.as_deref(), Some("pro"));
    assert_eq!(snapshot.model_context_window, Some(258_400));
}

#[test]
fn ignores_non_token_count_events() {
    let line = r#"{"timestamp":"2026-04-14T08:22:00.000Z","type":"response_item","payload":{"type":"message"}}"#;

    assert_eq!(parse_usage_snapshot(line, "session-123"), None);
}
