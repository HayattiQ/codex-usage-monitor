use crate::{model::UsageSnapshot, parser::parse_usage_snapshot};
use anyhow::Context;
use chrono::{DateTime, Utc};
use glob::glob;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug, Clone)]
pub struct SourcePoller {
    codex_dir: PathBuf,
    checkpoints: HashMap<String, u64>,
    mtimes: HashMap<String, SystemTime>,
}

#[derive(Debug, Default)]
pub struct PollResult {
    pub snapshots: Vec<UsageSnapshot>,
    pub parse_errors: usize,
    pub files_seen: usize,
    pub latest_event_at: Option<DateTime<Utc>>,
}

impl SourcePoller {
    pub fn new(codex_dir: PathBuf) -> Self {
        Self {
            codex_dir,
            checkpoints: HashMap::new(),
            mtimes: HashMap::new(),
        }
    }

    pub fn with_checkpoints(codex_dir: PathBuf, checkpoints: HashMap<String, u64>) -> Self {
        Self {
            codex_dir,
            checkpoints,
            mtimes: HashMap::new(),
        }
    }

    pub fn checkpoints(&self) -> &HashMap<String, u64> {
        &self.checkpoints
    }

    pub fn poll(&mut self) -> anyhow::Result<PollResult> {
        let sessions_dir = self.codex_dir.join("sessions");
        if !sessions_dir.exists() {
            return Ok(PollResult::default());
        }

        let pattern = format!("{}/**/*.jsonl", sessions_dir.display());
        let mut paths = glob(&pattern)
            .context("failed to glob session files")?
            .collect::<Result<Vec<_>, _>>()
            .context("failed to read session file path")?;
        paths.sort();

        let mut result = PollResult {
            files_seen: paths.len(),
            ..PollResult::default()
        };

        for path in paths {
            self.poll_file(&path, &mut result)?;
        }

        result.latest_event_at = result.snapshots.iter().map(|item| item.observed_at).max();
        Ok(result)
    }

    fn poll_file(&mut self, path: &Path, result: &mut PollResult) -> anyhow::Result<()> {
        let key = path.to_string_lossy().into_owned();
        let mut file =
            File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
        let metadata = file
            .metadata()
            .with_context(|| format!("failed to stat {}", path.display()))?;
        let current_len = metadata.len();
        let modified_at = metadata.modified().ok();
        let mut offset = self.checkpoints.get(&key).copied().unwrap_or(0);
        let changed_without_growth = modified_at
            .zip(self.mtimes.get(&key).copied())
            .is_some_and(|(current, previous)| current > previous && current_len <= offset);

        if current_len < offset || changed_without_growth {
            offset = 0;
        }

        file.seek(SeekFrom::Start(offset))
            .with_context(|| format!("failed to seek {}", path.display()))?;

        let mut new_content = String::new();
        file.read_to_string(&mut new_content)
            .with_context(|| format!("failed to read {}", path.display()))?;

        let session_id = session_id_from_path(path);
        for line in new_content.lines().filter(|line| !line.trim().is_empty()) {
            match parse_usage_snapshot(line, &session_id) {
                Some(snapshot) => result.snapshots.push(snapshot),
                None => {
                    if serde_json::from_str::<serde_json::Value>(line).is_err() {
                        result.parse_errors += 1;
                    }
                }
            }
        }

        self.checkpoints
            .insert(key, offset + new_content.len() as u64);
        if let Some(modified_at) = modified_at {
            self.mtimes
                .insert(path.to_string_lossy().into_owned(), modified_at);
        }
        Ok(())
    }
}

fn session_id_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("unknown-session")
        .to_owned()
}
