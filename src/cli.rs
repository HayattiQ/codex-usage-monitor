use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "cxusage")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long, global = true)]
    pub codex_dir: Option<PathBuf>,

    #[arg(long, global = true)]
    pub data_dir: Option<PathBuf>,

    #[arg(long, default_value = "30s", global = true)]
    pub interval: String,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Watch,
    Doctor,
}
