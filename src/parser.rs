use crate::model::{UsageSnapshot, UsageWindow};
use chrono::{DateTime, Utc};
use serde::Deserialize;

pub fn parse_usage_snapshot(line: &str, session_id: &str) -> Option<UsageSnapshot> {
    let event: SessionEvent = serde_json::from_str(line).ok()?;
    let payload = event.payload?;

    if payload.kind != "token_count" {
        return None;
    }

    let info = payload.info?;
    let rate_limits = payload.rate_limits?;

    Some(UsageSnapshot {
        observed_at: event.timestamp,
        session_id: session_id.to_owned(),
        primary: rate_limits.primary.into(),
        secondary: rate_limits.secondary.into(),
        plan_type: rate_limits.plan_type,
        model_context_window: info.model_context_window,
    })
}

#[derive(Debug, Deserialize)]
struct SessionEvent {
    timestamp: DateTime<Utc>,
    payload: Option<EventPayload>,
}

#[derive(Debug, Deserialize)]
struct EventPayload {
    #[serde(rename = "type")]
    kind: String,
    info: Option<EventInfo>,
    rate_limits: Option<RateLimits>,
}

#[derive(Debug, Deserialize)]
struct EventInfo {
    #[serde(rename = "total_token_usage")]
    _usage: TotalTokenUsage,
    model_context_window: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TotalTokenUsage {
    #[allow(dead_code)]
    total_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RateLimits {
    primary: RawWindow,
    secondary: RawWindow,
    plan_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawWindow {
    used_percent: Option<f64>,
    window_minutes: Option<u64>,
    resets_at: Option<i64>,
}

impl From<RawWindow> for UsageWindow {
    fn from(value: RawWindow) -> Self {
        Self {
            used_percent: value.used_percent,
            window_minutes: value.window_minutes,
            resets_at: value
                .resets_at
                .and_then(|unix| DateTime::<Utc>::from_timestamp(unix, 0)),
        }
    }
}
