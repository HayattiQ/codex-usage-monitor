use clap::Parser;
use cxusage::cli::{Cli, Command};

#[test]
fn parses_watch_command_with_default_interval() {
    let cli = Cli::parse_from(["cxusage", "watch"]);

    assert!(matches!(cli.command, Command::Watch));
    assert_eq!(cli.interval, "30s");
    assert!(cli.codex_dir.is_none());
    assert!(cli.data_dir.is_none());
}

#[test]
fn parses_doctor_command_with_overrides() {
    let cli = Cli::parse_from([
        "cxusage",
        "--codex-dir",
        "/tmp/.codex",
        "--data-dir",
        "/tmp/cxusage",
        "--interval",
        "2s",
        "doctor",
    ]);

    assert!(matches!(cli.command, Command::Doctor));
    assert_eq!(cli.interval, "2s");
    assert_eq!(cli.codex_dir.unwrap().to_string_lossy(), "/tmp/.codex");
    assert_eq!(cli.data_dir.unwrap().to_string_lossy(), "/tmp/cxusage");
}
