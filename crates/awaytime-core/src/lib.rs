//! Pure decisions: callers supply time, commands, observations, and data.
pub mod engine;
pub mod snapshot;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_INTEGER: u64 = 9_007_199_254_740_991;
pub type Extensions = BTreeMap<String, Value>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub work_ms: u64,
    pub short_break_ms: u64,
    #[serde(default)]
    pub long_break_ms: Option<u64>,
    #[serde(default)]
    pub long_every: Option<u64>,
    #[serde(default)]
    pub snooze_limit: Option<u64>,
    #[serde(flatten)]
    pub extra: Extensions,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            work_ms: 1_200_000,
            short_break_ms: 20_000,
            long_break_ms: None,
            long_every: None,
            snooze_limit: None,
            extra: Extensions::new(),
        }
    }
}
fn integer(value: Option<&Value>, positive: bool) -> bool {
    value
        .and_then(Value::as_u64)
        .is_some_and(|n| n <= MAX_INTEGER && (!positive || n > 0))
}
fn optional(value: Option<&Value>) -> bool {
    value.is_none_or(Value::is_null)
}

/// Sorted field errors, including type errors; bool and fractional values are never integers.
pub fn settings_errors(value: &Value) -> Vec<String> {
    let mut errors = BTreeSet::new();
    for field in ["work_ms", "short_break_ms"] {
        if !integer(value.get(field), true) {
            errors.insert(field.to_string());
        }
    }
    if !optional(value.get("snooze_limit")) && !integer(value.get("snooze_limit"), false) {
        errors.insert("snooze_limit".into());
    }
    let duration = value.get("long_break_ms");
    let every = value.get("long_every");
    if !optional(duration) || !optional(every) {
        if !integer(duration, true) {
            errors.insert("long_break_ms".into());
        }
        if !integer(every, true) {
            errors.insert("long_every".into());
        }
    }
    errors.into_iter().collect()
}
impl Settings {
    pub fn from_value(value: Value) -> Result<Self, String> {
        let errors = settings_errors(&value);
        if !errors.is_empty() {
            return Err(format!("Invalid settings: {}", errors.join(", ")));
        }
        serde_json::from_value(value).map_err(|e| e.to_string())
    }
    pub fn validate(&self) -> Result<(), String> {
        let value = serde_json::json!({"work_ms": self.work_ms, "short_break_ms": self.short_break_ms,
            "long_break_ms": self.long_break_ms, "long_every": self.long_every, "snooze_limit": self.snooze_limit});
        let errors = settings_errors(&value);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("Invalid settings: {}", errors.join(", ")))
        }
    }
}
#[derive(Debug, PartialEq, Serialize)]
pub struct PauseDecision {
    pub allowed: bool,
    pub reasons: Vec<String>,
}
pub fn pause_gate(reasons: &[String], excluded: &[String]) -> PauseDecision {
    let exclusions: BTreeSet<_> = excluded.iter().collect();
    let reasons: Vec<_> = reasons
        .iter()
        .filter(|r| !exclusions.contains(r))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    PauseDecision {
        allowed: reasons.is_empty(),
        reasons,
    }
}

pub fn valid_day(day: &str) -> bool {
    let parts: Vec<_> = day.split('-').collect();
    if parts.len() != 3
        || parts[0].len() != 4
        || parts[1].len() != 2
        || parts[2].len() != 2
        || !parts.iter().all(|p| p.bytes().all(|b| b.is_ascii_digit()))
    {
        return false;
    }
    let (Ok(y), Ok(m), Ok(d)) = (
        parts[0].parse::<u32>(),
        parts[1].parse::<u32>(),
        parts[2].parse::<u32>(),
    ) else {
        return false;
    };
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let max = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        _ => 0,
    };
    y > 0 && d > 0 && d <= max
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn invalid_numbers_and_half_long_break_are_rejected() {
        for bad in [
            json!(true),
            json!(1.5),
            json!(-1),
            json!(MAX_INTEGER + 1),
            json!("20"),
        ] {
            let mut value = serde_json::to_value(Settings::default()).unwrap();
            value["work_ms"] = bad;
            assert!(settings_errors(&value).contains(&"work_ms".into()));
        }
        let mut value = serde_json::to_value(Settings::default()).unwrap();
        value["long_every"] = json!(3);
        assert_eq!(settings_errors(&value), ["long_break_ms"]);
    }
    #[test]
    fn independent_reasons_and_calendar_boundaries() {
        let reasons = vec!["meeting".into(), "video".into(), "meeting".into()];
        assert_eq!(pause_gate(&reasons, &["meeting".into()]).reasons, ["video"]);
        assert!(valid_day("2024-02-29"));
        for bad in ["2025-02-29", "2026-13-01", "2026-01-00", "0000-01-01"] {
            assert!(!valid_day(bad));
        }
    }
}
