use chrono::{Duration, TimeZone, Utc};
use cxusage::{
    model::{UsageSnapshot, UsageWindow},
    store::HistoryStore,
};
use tempfile::tempdir;

#[test]
fn appends_snapshots_and_loads_recent_history() {
    let temp = tempdir().unwrap();
    let store = HistoryStore::new(temp.path().to_path_buf());

    let older = snapshot_at(
        Utc.with_ymd_and_hms(2026, 4, 13, 8, 0, 0).unwrap(),
        "session-a",
        5.0,
    );
    let newer = snapshot_at(
        Utc.with_ymd_and_hms(2026, 4, 14, 8, 0, 0).unwrap(),
        "session-b",
        15.0,
    );

    store.append_snapshot(&older).unwrap();
    store.append_snapshot(&newer).unwrap();

    let history = store
        .load_recent_snapshots(
            Utc.with_ymd_and_hms(2026, 4, 14, 12, 0, 0).unwrap(),
            Duration::hours(24),
        )
        .unwrap();

    assert_eq!(history, vec![newer]);
}

#[test]
fn saves_and_loads_checkpoints() {
    let temp = tempdir().unwrap();
    let store = HistoryStore::new(temp.path().to_path_buf());
    let checkpoints = [
        ("a.jsonl".to_string(), 12_u64),
        ("b.jsonl".to_string(), 30_u64),
    ]
    .into_iter()
    .collect();

    store.save_checkpoints(&checkpoints).unwrap();

    let loaded = store.load_checkpoints().unwrap();
    assert_eq!(loaded, checkpoints);
}

fn snapshot_at(
    observed_at: chrono::DateTime<Utc>,
    session_id: &str,
    primary_percent: f64,
) -> UsageSnapshot {
    UsageSnapshot {
        observed_at,
        session_id: session_id.to_owned(),
        primary: UsageWindow {
            used_percent: Some(primary_percent),
            window_minutes: Some(300),
            resets_at: Some(observed_at + Duration::hours(1)),
        },
        secondary: UsageWindow {
            used_percent: Some(primary_percent + 10.0),
            window_minutes: Some(10_080),
            resets_at: Some(observed_at + Duration::hours(2)),
        },
        plan_type: Some("pro".to_string()),
        model_context_window: Some(258_400),
    }
}
