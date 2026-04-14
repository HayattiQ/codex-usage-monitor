use crate::{
    cli::{Cli, Command},
    model::{UsageSnapshot, UsageWindow},
    source::{PollResult, SourcePoller},
    store::HistoryStore,
};
use anyhow::{Context, bail};
use chrono::{DateTime, Duration as ChronoDuration, FixedOffset, Local, Utc};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Sparkline},
};
use std::{
    io,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub codex_dir: PathBuf,
    pub data_dir: PathBuf,
    pub interval: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoctorReport {
    pub codex_dir_exists: bool,
    pub sessions_dir_exists: bool,
    pub files_seen: usize,
    pub latest_event_at: Option<DateTime<Utc>>,
    pub parse_errors: usize,
    pub checkpoints_count: usize,
}

#[derive(Debug, Clone)]
pub struct WatchState {
    pub latest: Option<UsageSnapshot>,
    pub history: Vec<UsageSnapshot>,
    pub files_seen: usize,
    pub parse_errors: usize,
    pub interval: Duration,
}

impl WatchState {
    pub fn last_updated(&self) -> Option<DateTime<Utc>> {
        self.latest.as_ref().map(|snapshot| snapshot.observed_at)
    }

    pub fn is_stale(&self, now: DateTime<Utc>) -> bool {
        let Some(last_updated) = self.last_updated() else {
            return true;
        };
        let threshold = ChronoDuration::from_std(self.interval.saturating_mul(2))
            .unwrap_or_else(|_| ChronoDuration::seconds(10));
        now - last_updated > threshold
    }
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = resolve_config(&cli)?;

    match cli.command {
        Command::Doctor => {
            let report = collect_doctor_report(config.codex_dir, config.data_dir)?;
            print_doctor_report(&report);
            Ok(())
        }
        Command::Watch => run_watch(config),
    }
}

pub fn resolve_config(cli: &Cli) -> anyhow::Result<AppConfig> {
    let codex_dir = match &cli.codex_dir {
        Some(path) => path.clone(),
        None => dirs::home_dir()
            .context("could not determine home directory")?
            .join(".codex"),
    };

    let data_dir = match &cli.data_dir {
        Some(path) => path.clone(),
        None => dirs::data_local_dir()
            .or_else(dirs::data_dir)
            .context("could not determine data directory")?
            .join("cxusage"),
    };

    Ok(AppConfig {
        codex_dir,
        data_dir,
        interval: parse_interval(&cli.interval)?,
    })
}

pub fn collect_doctor_report(
    codex_dir: PathBuf,
    data_dir: PathBuf,
) -> anyhow::Result<DoctorReport> {
    let store = HistoryStore::new(data_dir);
    let checkpoints = store.load_checkpoints()?;
    let mut poller = SourcePoller::new(codex_dir.clone());
    let poll = poller.poll()?;

    Ok(DoctorReport {
        codex_dir_exists: codex_dir.exists(),
        sessions_dir_exists: codex_dir.join("sessions").exists(),
        files_seen: poll.files_seen,
        latest_event_at: poll.latest_event_at,
        parse_errors: poll.parse_errors,
        checkpoints_count: checkpoints.len(),
    })
}

pub fn refresh_watch_state(
    poller: &mut SourcePoller,
    store: &HistoryStore,
    interval: Duration,
) -> anyhow::Result<WatchState> {
    let poll = poller.poll()?;
    for snapshot in &poll.snapshots {
        store.append_snapshot(snapshot)?;
    }
    store.save_checkpoints(poller.checkpoints())?;

    let mut history = store.load_recent_snapshots(Utc::now(), ChronoDuration::hours(24))?;
    history.sort_by_key(|snapshot| snapshot.observed_at);
    let latest = latest_snapshot(&poll, &history);

    Ok(WatchState {
        latest,
        history,
        files_seen: poll.files_seen,
        parse_errors: poll.parse_errors,
        interval,
    })
}

fn run_watch(config: AppConfig) -> anyhow::Result<()> {
    let store = HistoryStore::new(config.data_dir);
    let checkpoints = store.load_checkpoints()?;
    let mut poller = SourcePoller::with_checkpoints(config.codex_dir, checkpoints);
    let mut state = refresh_watch_state(&mut poller, &store, config.interval)?;
    let mut last_poll = Instant::now();

    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("failed to create terminal")?;

    let result = loop {
        terminal
            .draw(|frame| render_watch(frame, &state))
            .context("failed to draw terminal")?;

        if event::poll(Duration::from_millis(200)).context("failed to poll terminal event")?
            && let Event::Key(key) = event::read().context("failed to read terminal event")?
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            break Ok(());
        }

        if last_poll.elapsed() >= config.interval {
            state = refresh_watch_state(&mut poller, &store, config.interval)?;
            last_poll = Instant::now();
        }
    };

    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("failed to leave alternate screen")?;
    terminal.show_cursor().context("failed to show cursor")?;
    result
}

fn latest_snapshot(poll: &PollResult, history: &[UsageSnapshot]) -> Option<UsageSnapshot> {
    poll.snapshots
        .iter()
        .chain(history.iter())
        .max_by_key(|snapshot| snapshot.observed_at)
        .cloned()
}

fn render_watch(frame: &mut Frame<'_>, state: &WatchState) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(watch_layout_constraints())
        .split(area);

    let current = Text::from(current_lines(state));
    frame.render_widget(
        Paragraph::new(current).block(Block::default().title("cxusage").borders(Borders::ALL)),
        chunks[0],
    );

    let trend: Vec<u64> = state
        .history
        .iter()
        .filter_map(|snapshot| snapshot.primary.used_percent)
        .map(left_percent)
        .map(|value| value.round().clamp(0.0, 100.0) as u64)
        .collect();

    frame.render_widget(
        Sparkline::default()
            .block(
                Block::default()
                    .title("24h 5h limit left")
                    .borders(Borders::ALL),
            )
            .data(&trend)
            .max(100)
            .style(Style::default().fg(Color::Cyan)),
        chunks[1],
    );
}

pub fn watch_layout_constraints() -> Vec<Constraint> {
    vec![Constraint::Length(10), Constraint::Min(7)]
}

fn current_lines(state: &WatchState) -> Vec<Line<'static>> {
    watch_text_lines(state, Utc::now())
        .into_iter()
        .map(Line::from)
        .collect()
}

pub fn watch_text_lines(state: &WatchState, now: DateTime<Utc>) -> Vec<String> {
    let stale = if state.is_stale(now) { "stale" } else { "live" };
    let Some(snapshot) = &state.latest else {
        return vec![
            "status: stale".to_string(),
            "latest: unknown".to_string(),
            format!("files seen: {}", state.files_seen),
            format!("parse errors: {}", state.parse_errors),
        ];
    };

    vec![
        format!("status: {stale}"),
        format!(
            "plan: {}",
            snapshot.plan_type.as_deref().unwrap_or("unknown")
        ),
        format!(
            "last event: {}",
            format_datetime_local(snapshot.observed_at)
        ),
        format!("5h limit: {}", format_window_left(&snapshot.primary)),
        format!("weekly limit: {}", format_window_left(&snapshot.secondary)),
        format!(
            "context window: {}",
            snapshot
                .model_context_window
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ),
    ]
}

fn format_window_left(window: &UsageWindow) -> String {
    let left = window
        .used_percent
        .map(left_percent)
        .map(|value| format!("{value:.1}% left"))
        .unwrap_or_else(|| "unknown".to_string());
    let used = window
        .used_percent
        .map(|value| format!("{value:.1}% used"))
        .unwrap_or_else(|| "unknown used".to_string());
    let window_minutes = window
        .window_minutes
        .map(format_window_minutes)
        .unwrap_or_else(|| "unknown window".to_string());
    let resets_at = window
        .resets_at
        .map(format_datetime_local)
        .unwrap_or_else(|| "unknown reset".to_string());

    format!("{left} ({used}) / {window_minutes}, resets {resets_at}")
}

fn left_percent(used_percent: f64) -> f64 {
    (100.0 - used_percent).clamp(0.0, 100.0)
}

fn format_window_minutes(minutes: u64) -> String {
    match minutes {
        300 => "5h".to_string(),
        10_080 => "weekly".to_string(),
        value if value % 60 == 0 => format!("{}h", value / 60),
        value => format!("{value}m"),
    }
}

fn print_doctor_report(report: &DoctorReport) {
    println!("codex dir: {}", ok(report.codex_dir_exists));
    println!("sessions dir: {}", ok(report.sessions_dir_exists));
    println!("session files: {}", report.files_seen);
    println!(
        "latest event: {}",
        report
            .latest_event_at
            .map(format_datetime_local)
            .unwrap_or_else(|| "none".to_string())
    );
    println!("parse errors: {}", report.parse_errors);
    println!("checkpoints: {}", report.checkpoints_count);
}

fn ok(value: bool) -> &'static str {
    if value { "ok" } else { "missing" }
}

fn format_datetime_local(value: DateTime<Utc>) -> String {
    value
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S %:z")
        .to_string()
}

pub fn format_datetime_with_offset(value: DateTime<Utc>, offset: FixedOffset) -> String {
    value
        .with_timezone(&offset)
        .format("%Y-%m-%d %H:%M:%S %:z")
        .to_string()
}

fn parse_interval(value: &str) -> anyhow::Result<Duration> {
    let Some((number, suffix)) = value
        .trim()
        .find(|character: char| !character.is_ascii_digit())
        .map(|index| value.trim().split_at(index))
    else {
        let seconds = value
            .trim()
            .parse::<u64>()
            .context("invalid interval seconds")?;
        return Ok(Duration::from_secs(seconds));
    };

    let amount = number.parse::<u64>().context("invalid interval amount")?;
    match suffix {
        "s" => Ok(Duration::from_secs(amount)),
        "m" => Ok(Duration::from_secs(amount * 60)),
        "h" => Ok(Duration::from_secs(amount * 60 * 60)),
        _ => bail!("unsupported interval suffix: {suffix}"),
    }
}
