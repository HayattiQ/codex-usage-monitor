use crate::model::UsageSnapshot;
use anyhow::Context;
use chrono::{DateTime, Duration, Utc};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub struct HistoryStore {
    root: PathBuf,
}

impl HistoryStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn append_snapshot(&self, snapshot: &UsageSnapshot) -> anyhow::Result<()> {
        self.ensure_root()?;
        let history_path = self.history_path();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&history_path)
            .context("failed to open history file")?;
        serde_json::to_writer(&mut file, snapshot).context("failed to serialize snapshot")?;
        writeln!(file).context("failed to write history line")?;
        Ok(())
    }

    pub fn load_recent_snapshots(
        &self,
        now: DateTime<Utc>,
        window: Duration,
    ) -> anyhow::Result<Vec<UsageSnapshot>> {
        let path = self.history_path();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let cutoff = now - window;
        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .with_context(|| format!("failed to open {}", path.display()))?;
        let reader = BufReader::new(file);
        let mut snapshots = Vec::new();

        for line in reader.lines() {
            let line = line.context("failed to read history line")?;
            let snapshot: UsageSnapshot =
                serde_json::from_str(&line).context("failed to parse history snapshot")?;
            if snapshot.observed_at >= cutoff {
                snapshots.push(snapshot);
            }
        }

        Ok(snapshots)
    }

    pub fn save_checkpoints(&self, checkpoints: &HashMap<String, u64>) -> anyhow::Result<()> {
        self.ensure_root()?;
        let body =
            serde_json::to_vec_pretty(checkpoints).context("failed to serialize checkpoints")?;
        fs::write(self.checkpoints_path(), body).context("failed to write checkpoints")?;
        Ok(())
    }

    pub fn load_checkpoints(&self) -> anyhow::Result<HashMap<String, u64>> {
        let path = self.checkpoints_path();
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let body = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let checkpoints =
            serde_json::from_str(&body).context("failed to parse checkpoints file")?;
        Ok(checkpoints)
    }

    pub fn history_path(&self) -> PathBuf {
        self.root.join("history.jsonl")
    }

    fn checkpoints_path(&self) -> PathBuf {
        self.root.join("checkpoints.json")
    }

    fn ensure_root(&self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.root)
            .with_context(|| format!("failed to create {}", self.root.display()))?;
        Ok(())
    }
}
