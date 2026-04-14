use cxusage::source::SourcePoller;
use std::fs;
use tempfile::tempdir;

#[test]
fn polls_only_new_lines_and_updates_offsets() {
    let temp = tempdir().unwrap();
    let codex_dir = temp.path().join(".codex");
    let session_dir = codex_dir.join("sessions/2026/04/14");
    fs::create_dir_all(&session_dir).unwrap();

    let session_file = session_dir.join("rollout.jsonl");
    fs::write(
        &session_file,
        format!("{}\n", usage_line("2026-04-14T08:22:00.000Z", 8.0, 56.0)),
    )
    .unwrap();

    let mut poller = SourcePoller::new(codex_dir.clone());
    let first = poller.poll().unwrap();
    assert_eq!(first.snapshots.len(), 1);
    assert_eq!(first.snapshots[0].primary.used_percent, Some(8.0));

    fs::write(
        &session_file,
        format!(
            "{}\n{}\n",
            usage_line("2026-04-14T08:22:00.000Z", 8.0, 56.0),
            usage_line("2026-04-14T08:27:00.000Z", 12.0, 57.0)
        ),
    )
    .unwrap();

    let second = poller.poll().unwrap();
    assert_eq!(second.snapshots.len(), 1);
    assert_eq!(second.snapshots[0].primary.used_percent, Some(12.0));
    assert_eq!(second.parse_errors, 0);
}

#[test]
fn resets_offset_when_file_shrinks() {
    let temp = tempdir().unwrap();
    let codex_dir = temp.path().join(".codex");
    let session_dir = codex_dir.join("sessions/2026/04/14");
    fs::create_dir_all(&session_dir).unwrap();

    let session_file = session_dir.join("rollout.jsonl");
    fs::write(
        &session_file,
        format!(
            "{}\n{}\n",
            usage_line("2026-04-14T08:22:00.000Z", 8.0, 56.0),
            usage_line("2026-04-14T08:24:00.000Z", 9.0, 56.5)
        ),
    )
    .unwrap();

    let mut poller = SourcePoller::new(codex_dir.clone());
    let _ = poller.poll().unwrap();

    fs::write(
        &session_file,
        format!("{}\n", usage_line("2026-04-14T08:30:00.000Z", 3.0, 10.0)),
    )
    .unwrap();

    let after_truncate = poller.poll().unwrap();
    assert_eq!(after_truncate.snapshots.len(), 1);
    assert_eq!(after_truncate.snapshots[0].primary.used_percent, Some(3.0));
}

fn usage_line(timestamp: &str, primary: f64, secondary: f64) -> String {
    format!(
        r#"{{"timestamp":"{timestamp}","type":"event_msg","payload":{{"type":"token_count","info":{{"total_token_usage":{{"total_tokens":1110}},"last_token_usage":{{"total_tokens":11}},"model_context_window":258400}},"rate_limits":{{"limit_id":"codex","limit_name":null,"primary":{{"used_percent":{primary},"window_minutes":300,"resets_at":1774351986}},"secondary":{{"used_percent":{secondary},"window_minutes":10080,"resets_at":1774457050}},"credits":null,"plan_type":"pro"}}}}}}"#
    )
}
