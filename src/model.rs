use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub observed_at: DateTime<Utc>,
    pub session_id: String,
    pub primary: UsageWindow,
    pub secondary: UsageWindow,
    pub plan_type: Option<String>,
    pub model_context_window: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UsageWindow {
    pub used_percent: Option<f64>,
    pub window_minutes: Option<u64>,
    pub resets_at: Option<DateTime<Utc>>,
}
