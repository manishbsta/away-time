use crate::{valid_day, Extensions, Settings, MAX_INTEGER};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Stopped,
    Working,
    Paused,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub phase: Phase,
    pub work_elapsed_ms: u64,
    pub cycle: u64,
    #[serde(default)]
    pub quota_day: Option<String>,
    pub snoozes_today: u64,
    pub snoozes_session: u64,
    #[serde(default)]
    pub pause_reasons: Vec<String>,
    #[serde(flatten)]
    pub extra: Extensions,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            phase: Phase::Stopped,
            work_elapsed_ms: 0,
            cycle: 0,
            quota_day: None,
            snoozes_today: 0,
            snoozes_session: 0,
            pause_reasons: vec![],
            extra: Extensions::new(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub schema_version: u64,
    pub settings: Settings,
    pub session: Session,
    #[serde(flatten)]
    pub extra: Extensions,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            schema_version: 2,
            settings: Settings::default(),
            session: Session::default(),
            extra: Extensions::new(),
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("Unsupported snapshot schema {0}")]
    Schema(u64),
    #[error("Invalid snapshot: {0}")]
    Invalid(String),
}
impl Snapshot {
    pub fn validate(&self) -> Result<(), CodecError> {
        if self.schema_version != 2 {
            return Err(CodecError::Schema(self.schema_version));
        }
        self.settings.validate().map_err(CodecError::Invalid)?;
        for value in [
            self.session.work_elapsed_ms,
            self.session.cycle,
            self.session.snoozes_today,
            self.session.snoozes_session,
        ] {
            if value > MAX_INTEGER {
                return Err(CodecError::Invalid("session integer overflow".into()));
            }
        }
        if self
            .session
            .quota_day
            .as_ref()
            .is_some_and(|d| !valid_day(d))
        {
            return Err(CodecError::Invalid("quota_day".into()));
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        self.validate()?;
        // Serialize known fields last so extension collisions never override them.
        let settings = merge_known(
            &self.settings.extra,
            serde_json::json!({
                "work_ms": self.settings.work_ms, "short_break_ms": self.settings.short_break_ms,
                "long_break_ms": self.settings.long_break_ms, "long_every": self.settings.long_every,
                "snooze_limit": self.settings.snooze_limit
            }),
        );
        let session = merge_known(
            &self.session.extra,
            serde_json::json!({
                "phase": self.session.phase, "work_elapsed_ms": self.session.work_elapsed_ms,
                "cycle": self.session.cycle, "quota_day": self.session.quota_day,
                "snoozes_today": self.session.snoozes_today, "snoozes_session": self.session.snoozes_session,
                "pause_reasons": self.session.pause_reasons
            }),
        );
        serde_json::to_vec_pretty(&merge_known(
            &self.extra,
            serde_json::json!({
                "schema_version": 2, "settings": settings, "session": session
            }),
        ))
        .map_err(|e| CodecError::Invalid(e.to_string()))
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        let mut value: Value =
            serde_json::from_slice(bytes).map_err(|e| CodecError::Invalid(e.to_string()))?;
        let version = value
            .get("schema_version")
            .and_then(Value::as_u64)
            .ok_or_else(|| CodecError::Invalid("schema_version".into()))?;
        match version {
            1 => {
                let settings = value
                    .get_mut("settings")
                    .and_then(Value::as_object_mut)
                    .ok_or_else(|| CodecError::Invalid("settings".into()))?;
                for (old, new) in [
                    ("work_seconds", "work_ms"),
                    ("short_break_seconds", "short_break_ms"),
                ] {
                    if settings.contains_key(new) {
                        return Err(CodecError::Invalid("ambiguous duration fields".into()));
                    }
                    let n = settings
                        .remove(old)
                        .and_then(|v| v.as_u64())
                        .and_then(|n| n.checked_mul(1000))
                        .filter(|n| *n > 0 && *n <= MAX_INTEGER)
                        .ok_or_else(|| CodecError::Invalid(old.into()))?;
                    settings.insert(new.into(), Value::from(n));
                }
                value["schema_version"] = Value::from(2);
            }
            2 => (),
            other => return Err(CodecError::Schema(other)),
        }
        let mut snapshot: Self =
            serde_json::from_value(value).map_err(|e| CodecError::Invalid(e.to_string()))?;
        snapshot.validate()?;
        snapshot.session.pause_reasons.sort();
        snapshot.session.pause_reasons.dedup();
        if snapshot.session.phase == Phase::Paused && snapshot.session.pause_reasons.is_empty() {
            snapshot.session.pause_reasons.push("manual".into());
        }
        Ok(snapshot)
    }
}
fn merge_known(extra: &Extensions, known: Value) -> Value {
    let mut result: Map<String, Value> = extra.clone().into_iter().collect();
    result.extend(known.as_object().unwrap().clone());
    Value::Object(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn migration_extensions_and_collisions_round_trip() {
        let mut old = serde_json::to_value(Snapshot::default()).unwrap();
        old["schema_version"] = json!(1);
        let settings = old["settings"].as_object_mut().unwrap();
        settings.remove("work_ms");
        settings.remove("short_break_ms");
        settings.insert("work_seconds".into(), json!(1200));
        settings.insert("short_break_seconds".into(), json!(20));
        settings.insert("future_setting".into(), json!({"enabled": true}));
        old["future_root"] = json!([1, 2]);
        old["session"]["future_session"] = json!("keep");
        old["session"]["phase"] = json!("paused");
        let mut snapshot = Snapshot::decode(&serde_json::to_vec(&old).unwrap()).unwrap();
        assert_eq!(snapshot.session.pause_reasons, ["manual"]);
        snapshot.settings.extra.insert("work_ms".into(), json!(1));
        snapshot.extra.insert("schema_version".into(), json!(99));
        let restored = Snapshot::decode(&snapshot.encode().unwrap()).unwrap();
        assert_eq!(restored.settings.work_ms, 1_200_000);
        assert_eq!(
            restored.settings.extra["future_setting"],
            json!({"enabled": true})
        );
        assert_eq!(restored.extra["future_root"], json!([1, 2]));
        assert_eq!(restored.session.extra["future_session"], "keep");
    }
    #[test]
    fn future_schema_and_incomplete_rest_are_errors() {
        let mut value = serde_json::to_value(Snapshot::default()).unwrap();
        value["schema_version"] = json!(3);
        assert!(matches!(
            Snapshot::decode(&serde_json::to_vec(&value).unwrap()),
            Err(CodecError::Schema(3))
        ));
        value["schema_version"] = json!(2);
        value["session"]["phase"] = json!("resting");
        assert!(Snapshot::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    }
}
