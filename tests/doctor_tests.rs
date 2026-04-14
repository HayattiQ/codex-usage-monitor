use chrono::{TimeZone, Utc};
use clap::Parser;
use cxusage::app::{collect_doctor_report, resolve_config};
use cxusage::cli::{Cli, Command};
use std::fs;
use tempfile::tempdir;

#[test]
fn doctor_report_reads_latest_usage_event_and_checkpoints() {
    let temp = tempdir().unwrap();
    let codex_dir = temp.path().join(".codex");
    let data_dir = temp.path().join("data");
    fs::create_dir_all(codex_dir.join("sessions/2026/04/14")).unwrap();
    fs::create_dir_all(&data_dir).unwrap();
    fs::write(
        codex_dir.join("sessions/2026/04/14/rollout.jsonl"),
        format!("{}\n", usage_line("2026-04-14T08:22:00.000Z", 8.0, 56.0)),
    )
    .unwrap();
    fs::write(
        data_dir.join("checkpoints.json"),
        r#"{"existing.jsonl":12}"#,
    )
    .unwrap();

    let report = collect_doctor_report(codex_dir, data_dir).unwrap();

    assert!(report.codex_dir_exists);
    assert!(report.sessions_dir_exists);
    assert_eq!(report.files_seen, 1);
    assert_eq!(report.parse_errors, 0);
    assert_eq!(report.checkpoints_count, 1);
    assert_eq!(
        report.latest_event_at,
        Some(Utc.with_ymd_and_hms(2026, 4, 14, 8, 22, 0).unwrap())
    );
}

#[test]
fn resolve_config_uses_cli_overrides() {
    let cli = Cli::parse_from([
        "cxusage",
        "--codex-dir",
        "/tmp/codex",
        "--data-dir",
        "/tmp/cxusage",
        "--interval",
        "7s",
        "doctor",
    ]);

    let config = resolve_config(&cli).unwrap();

    assert!(matches!(cli.command, Command::Doctor));
    assert_eq!(config.codex_dir.to_string_lossy(), "/tmp/codex");
    assert_eq!(config.data_dir.to_string_lossy(), "/tmp/cxusage");
    assert_eq!(config.interval.as_secs(), 7);
}

fn usage_line(timestamp: &str, primary: f64, secondary: f64) -> String {
    format!(
        r#"{{"timestamp":"{timestamp}","type":"event_msg","payload":{{"type":"token_count","info":{{"total_token_usage":{{"total_tokens":1110}},"last_token_usage":{{"total_tokens":11}},"model_context_window":258400}},"rate_limits":{{"limit_id":"codex","limit_name":null,"primary":{{"used_percent":{primary},"window_minutes":300,"resets_at":1774351986}},"secondary":{{"used_percent":{secondary},"window_minutes":10080,"resets_at":1774457050}},"credits":null,"plan_type":"pro"}}}}}}"#
    )
}
