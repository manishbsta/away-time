use crate::{
    snapshot::{Phase, Snapshot},
    valid_day, Settings, MAX_INTEGER,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockSample {
    pub monotonic_ms: u64,
    pub wall_clock_ms: i64,
    pub local_day: String,
    pub weekday: u8,
    pub minute: u16,
    pub timezone: String,
}
impl ClockSample {
    pub fn validate(&self) -> Result<(), String> {
        if self.monotonic_ms > MAX_INTEGER
            || self.wall_clock_ms.unsigned_abs() > MAX_INTEGER
            || !valid_day(&self.local_day)
            || self.weekday > 6
            || self.minute > 1439
            || self.timezone.is_empty()
        {
            return Err("Invalid clock sample".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "settings", rename_all = "snake_case")]
pub enum Command {
    Start,
    Stop,
    Pause,
    Resume,
    ReplaceSettings(Settings),
    PreviewBreak,
    EndPreview,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    Persist,
    ShowPreview,
    ClosePreview,
}
#[derive(Debug, Clone)]
pub struct CoreState {
    pub snapshot: Snapshot,
    pub last_sequence: u64,
    pub last_monotonic_ms: Option<u64>,
}
impl CoreState {
    pub fn restore(snapshot: Snapshot) -> Result<Self, String> {
        snapshot.validate().map_err(|e| e.to_string())?;
        Ok(Self {
            snapshot,
            last_sequence: 0,
            last_monotonic_ms: None,
        })
    }
    pub fn reduce(
        &self,
        sequence: u64,
        command: Command,
        clock: &ClockSample,
    ) -> Result<(Self, Vec<Effect>), String> {
        clock.validate()?;
        if sequence <= self.last_sequence {
            return Ok((self.clone(), vec![]));
        }
        if sequence > MAX_INTEGER {
            return Err("Command sequence overflow".into());
        }
        if self
            .last_monotonic_ms
            .is_some_and(|t| clock.monotonic_ms < t)
        {
            return Err("Elapsed time moved backward".into());
        }
        // Phase 0 records command state only. Interval scheduling belongs to Phase 1.
        let mut next = self.clone();
        next.last_sequence = sequence;
        next.last_monotonic_ms = Some(clock.monotonic_ms);
        let mut effects = vec![];
        match command {
            Command::Start => {
                if next.snapshot.session.phase == Phase::Stopped {
                    next.snapshot.session.phase = Phase::Working;
                    next.snapshot.session.pause_reasons.clear();
                }
            }
            Command::Stop => {
                next.snapshot.session.phase = Phase::Stopped;
                next.snapshot.session.pause_reasons.clear();
                effects.push(Effect::ClosePreview);
            }
            Command::Pause => {
                if next.snapshot.session.phase == Phase::Working {
                    next.snapshot.session.phase = Phase::Paused;
                    next.snapshot.session.pause_reasons.push("manual".into());
                }
            }
            Command::Resume => {
                next.snapshot
                    .session
                    .pause_reasons
                    .retain(|r| r != "manual");
                if next.snapshot.session.phase == Phase::Paused
                    && next.snapshot.session.pause_reasons.is_empty()
                {
                    next.snapshot.session.phase = Phase::Working;
                }
            }
            Command::ReplaceSettings(settings) => {
                settings.validate()?;
                next.snapshot.settings = settings;
            }
            Command::PreviewBreak => effects.push(Effect::ShowPreview),
            Command::EndPreview => effects.push(Effect::ClosePreview),
        }
        if next.snapshot != self.snapshot {
            effects.push(Effect::Persist);
        }
        Ok((next, effects))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn clock(ms: u64, wall: i64) -> ClockSample {
        ClockSample {
            monotonic_ms: ms,
            wall_clock_ms: wall,
            local_day: "2026-10-13".into(),
            weekday: 1,
            minute: 500,
            timezone: "Asia/Kathmandu".into(),
        }
    }
    #[test]
    fn stale_commands_backwards_time_and_wall_changes() {
        let state = CoreState::restore(Snapshot::default()).unwrap();
        let (started, effects) = state.reduce(1, Command::Start, &clock(100, 1000)).unwrap();
        assert_eq!(effects, [Effect::Persist]);
        let (stale, effects) = started.reduce(1, Command::Stop, &clock(100, 1000)).unwrap();
        assert_eq!(stale.snapshot, started.snapshot);
        assert!(effects.is_empty());
        assert!(started.reduce(2, Command::Pause, &clock(99, 1000)).is_err());
        let (paused, _) = started
            .reduce(2, Command::Pause, &clock(200, -1000))
            .unwrap();
        assert_eq!(paused.snapshot.session.phase, Phase::Paused);
        let restored = CoreState::restore(paused.snapshot).unwrap();
        assert_eq!(restored.last_monotonic_ms, None);
        assert_eq!(restored.last_sequence, 0);
    }
    #[test]
    fn manual_resume_cannot_clear_other_pause_reasons() {
        let mut snapshot = Snapshot::default();
        snapshot.session.phase = Phase::Paused;
        snapshot.session.pause_reasons = vec!["manual".into(), "meeting".into()];
        let state = CoreState::restore(snapshot).unwrap();
        let (next, _) = state.reduce(1, Command::Resume, &clock(0, 0)).unwrap();
        assert_eq!(next.snapshot.session.phase, Phase::Paused);
        assert_eq!(next.snapshot.session.pause_reasons, ["meeting"]);
    }
}
