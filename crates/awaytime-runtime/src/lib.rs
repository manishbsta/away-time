use awaytime_core::{
    engine::{ClockSample, Command, CoreState, Effect},
    snapshot::Snapshot,
    MAX_INTEGER,
};
use serde::Serialize;
use std::{collections::BTreeMap, sync::Mutex};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", content = "detail", rename_all = "snake_case")]
pub enum Capability {
    Supported,
    Unavailable(String),
    PermissionRequired(String),
    PermissionDenied(String),
}
#[derive(Debug, Clone, Copy)]
pub enum Source {
    Ui,
    Tray,
    Keyboard,
    Script,
}
pub trait ClockSource: Send + Sync {
    fn sample(&self) -> Result<ClockSample, String>;
}
pub trait ActivitySource: Send {
    fn active_reasons(&mut self) -> Result<Vec<String>, String>;
}
pub trait IdleSource: Send {
    fn idle_ms(&mut self) -> Result<u64, String>;
}
pub trait DisplaySource: Send {
    fn display_ids(&self) -> Result<Vec<String>, String>;
}
pub trait NotificationSink: Send {
    fn show(&mut self, id: &str, message: &str) -> Result<(), String>;
    fn dismiss_all(&mut self);
}
pub trait AudioSink: Send {
    fn play(&mut self, id: &str) -> Result<(), String>;
    fn stop_all(&mut self);
}
pub trait SnapshotStore: Send {
    fn load(&mut self) -> Result<Option<Snapshot>, String>;
    fn save(&mut self, snapshot: &Snapshot) -> Result<(), String>;
}
pub trait FocusSource: Send {
    fn active(&mut self) -> Result<bool, String>;
}
pub trait CalendarSource: Send {
    fn pause_reasons(&mut self) -> Result<Vec<String>, String>;
}
pub trait AutomationRunner: Send {
    fn run(&mut self, break_id: &str, action_id: &str) -> Result<(), String>;
    fn cancel_all(&mut self);
}
pub trait ScriptSource: Send {
    fn next_command(&mut self) -> Result<Option<Command>, String>;
}

/// One synchronized command path for UI, tray, keyboard, and script callers.
pub struct Gateway(Mutex<CoreState>);
impl Gateway {
    pub fn new(snapshot: Snapshot) -> Result<Self, String> {
        Ok(Self(Mutex::new(CoreState::restore(snapshot)?)))
    }
    pub fn snapshot(&self) -> Result<Snapshot, String> {
        Ok(self
            .0
            .lock()
            .map_err(|_| "Command state unavailable")?
            .snapshot
            .clone())
    }
    pub fn dispatch(
        &self,
        command: Command,
        _source: Source,
        clock: &ClockSample,
    ) -> Result<(Snapshot, Vec<Effect>), String> {
        let mut state = self.0.lock().map_err(|_| "Command state unavailable")?;
        let sequence = state
            .last_sequence
            .checked_add(1)
            .filter(|n| *n <= MAX_INTEGER)
            .ok_or("Command sequence overflow")?;
        let (next, effects) = state.reduce(sequence, command, clock)?;
        let snapshot = next.snapshot.clone();
        *state = next;
        Ok((snapshot, effects))
    }
}

/// Own a resource exactly once. Registering after shutdown immediately releases it.
#[derive(Default)]
pub struct ResourceScope {
    closed: bool,
    resources: BTreeMap<String, Box<dyn FnOnce() + Send>>,
}
impl ResourceScope {
    pub fn register(&mut self, id: String, release: impl FnOnce() + Send + 'static) {
        if self.closed {
            release();
            return;
        }
        if let Some(old) = self.resources.insert(id, Box::new(release)) {
            old();
        }
    }
    pub fn close(&mut self) {
        self.closed = true;
        for release in std::mem::take(&mut self.resources).into_values() {
            release();
        }
    }
}
impl Drop for ResourceScope {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    fn clock() -> ClockSample {
        ClockSample {
            monotonic_ms: 0,
            wall_clock_ms: 0,
            local_day: "2026-10-13".into(),
            weekday: 1,
            minute: 0,
            timezone: "UTC".into(),
        }
    }
    #[test]
    fn action_sources_share_commands() {
        for source in [Source::Ui, Source::Tray, Source::Keyboard, Source::Script] {
            let gateway = Gateway::new(Snapshot::default()).unwrap();
            let (snapshot, effects) = gateway.dispatch(Command::Start, source, &clock()).unwrap();
            assert_eq!(
                snapshot.session.phase,
                awaytime_core::snapshot::Phase::Working
            );
            assert_eq!(effects, [Effect::Persist]);
        }
    }
    #[test]
    fn cleanup_is_once_including_replacements_and_late_resources() {
        let count = Arc::new(AtomicUsize::new(0));
        let mut scope = ResourceScope::default();
        for _ in 0..2 {
            let count = count.clone();
            scope.register("window".into(), move || {
                count.fetch_add(1, Ordering::SeqCst);
            });
        }
        scope.close();
        scope.close();
        let late = count.clone();
        scope.register("late".into(), move || {
            late.fetch_add(1, Ordering::SeqCst);
        });
        drop(scope);
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }
}
