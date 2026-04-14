use chrono::{FixedOffset, TimeZone, Utc};
use cxusage::{
    app::{WatchState, format_datetime_with_offset, watch_layout_constraints, watch_text_lines},
    model::{UsageSnapshot, UsageWindow},
};
use ratatui::layout::Constraint;
use std::time::Duration;

#[test]
fn watch_text_lines_show_unknown_fallbacks() {
    let state = WatchState {
        latest: Some(UsageSnapshot {
            observed_at: Utc.with_ymd_and_hms(2026, 4, 14, 8, 0, 0).unwrap(),
            session_id: "session-1".to_string(),
            primary: UsageWindow::default(),
            secondary: UsageWindow::default(),
            plan_type: None,
            model_context_window: None,
        }),
        history: Vec::new(),
        files_seen: 1,
        parse_errors: 0,
        interval: Duration::from_secs(5),
    };

    let lines = watch_text_lines(&state, Utc.with_ymd_and_hms(2026, 4, 14, 8, 0, 6).unwrap());

    assert!(lines.iter().any(|line| line == "plan: unknown"));
    assert!(lines.iter().any(|line| line.contains("5h limit: unknown")));
    assert!(
        lines
            .iter()
            .any(|line| line.contains("weekly limit: unknown"))
    );
    assert!(lines.iter().any(|line| line == "context window: unknown"));
}

#[test]
fn watch_title_mentions_quit_keys() {
    assert_eq!(cxusage::app::watch_title(), "cxusage (q/Esc to quit)");
}

#[test]
fn watch_text_lines_show_left_first_labels() {
    let state = WatchState {
        latest: Some(UsageSnapshot {
            observed_at: Utc.with_ymd_and_hms(2026, 4, 14, 8, 0, 0).unwrap(),
            session_id: "session-1".to_string(),
            primary: UsageWindow {
                used_percent: Some(8.0),
                window_minutes: Some(300),
                resets_at: Some(Utc.with_ymd_and_hms(2026, 4, 14, 9, 0, 0).unwrap()),
            },
            secondary: UsageWindow {
                used_percent: Some(56.0),
                window_minutes: Some(10_080),
                resets_at: Some(Utc.with_ymd_and_hms(2026, 4, 21, 8, 0, 0).unwrap()),
            },
            plan_type: Some("pro".to_string()),
            model_context_window: Some(258_400),
        }),
        history: Vec::new(),
        files_seen: 1,
        parse_errors: 0,
        interval: Duration::from_secs(5),
    };

    let lines = watch_text_lines(&state, Utc.with_ymd_and_hms(2026, 4, 14, 8, 0, 6).unwrap());

    assert!(
        lines
            .iter()
            .any(|line| line.contains("5h limit: 92.0% left"))
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("weekly limit: 44.0% left"))
    );
    assert!(!lines.iter().any(|line| line.contains("primary:")));
    assert!(!lines.iter().any(|line| line.contains("secondary:")));
}

#[test]
fn watch_text_lines_show_last_event_only() {
    let state = WatchState {
        latest: Some(UsageSnapshot {
            observed_at: Utc.with_ymd_and_hms(2026, 4, 14, 8, 0, 0).unwrap(),
            session_id: "session-1".to_string(),
            primary: UsageWindow::default(),
            secondary: UsageWindow::default(),
            plan_type: None,
            model_context_window: None,
        }),
        history: Vec::new(),
        files_seen: 1,
        parse_errors: 0,
        interval: Duration::from_secs(5),
    };

    let lines = watch_text_lines(&state, Utc.with_ymd_and_hms(2026, 4, 14, 8, 0, 11).unwrap());

    assert!(lines.iter().any(|line| line.starts_with("last event: ")));
    assert!(!lines.iter().any(|line| line.starts_with("last update:")));
}

#[test]
fn formats_timestamp_for_jst_offset() {
    let timestamp = Utc.with_ymd_and_hms(2026, 4, 14, 8, 0, 0).unwrap();
    let jst = FixedOffset::east_opt(9 * 60 * 60).unwrap();

    assert_eq!(
        format_datetime_with_offset(timestamp, jst),
        "2026-04-14 17:00:00 +09:00"
    );
}

#[test]
fn watch_state_is_stale_after_two_intervals() {
    let state = WatchState {
        latest: Some(UsageSnapshot {
            observed_at: Utc.with_ymd_and_hms(2026, 4, 14, 8, 0, 0).unwrap(),
            session_id: "session-1".to_string(),
            primary: UsageWindow::default(),
            secondary: UsageWindow::default(),
            plan_type: None,
            model_context_window: None,
        }),
        history: Vec::new(),
        files_seen: 1,
        parse_errors: 0,
        interval: Duration::from_secs(5),
    };

    assert!(!state.is_stale(Utc.with_ymd_and_hms(2026, 4, 14, 8, 0, 10).unwrap()));
    assert!(state.is_stale(Utc.with_ymd_and_hms(2026, 4, 14, 8, 0, 11).unwrap()));
}

#[test]
fn watch_layout_removes_help_and_reserves_more_usage_rows() {
    assert_eq!(
        watch_layout_constraints(),
        vec![Constraint::Length(10), Constraint::Min(7)]
    );
}
