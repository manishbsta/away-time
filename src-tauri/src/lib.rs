mod platform;
use awaytime_core::{
    engine::{ClockSample, Command, Effect},
    snapshot::{Phase, Snapshot},
};
use awaytime_persistence::FileStore;
use awaytime_runtime::{Capability, Gateway, ResourceScope, Source};
use chrono::{Datelike, Local, Timelike};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder,
};

type Shared = Arc<Mutex<ProofRuntime>>;
struct ProofRuntime {
    gateway: Gateway,
    store: FileStore,
    origin: Instant,
    last_tick: Instant,
    elapsed_ms: u64,
    last_wake: u64,
    preview_deadline: Option<Instant>,
    preview_generation: u64,
    preview_monitors: Vec<(i32, i32, u32, u32)>,
    foreground: usize,
    preview: ResourceScope,
    resources: ResourceScope,
    capabilities: BTreeMap<String, Capability>,
    storage_notice: Option<String>,
    last_error: Option<String>,
    storage_blocked: bool,
}
#[derive(Serialize)]
struct ProofStatus {
    phase: Phase,
    elapsed_ms: u64,
    idle_ms: Option<u64>,
    display_count: usize,
    preview_active: bool,
    preview_window_count: usize,
    wake_count: u64,
    capabilities: BTreeMap<String, Capability>,
    storage_notice: Option<String>,
    last_error: Option<String>,
}
impl ProofRuntime {
    fn clock(&self) -> ClockSample {
        let now = Local::now();
        ClockSample {
            monotonic_ms: self.origin.elapsed().as_millis() as u64,
            wall_clock_ms: now.timestamp_millis(),
            local_day: now.format("%Y-%m-%d").to_string(),
            weekday: now.weekday().num_days_from_monday() as u8,
            minute: (now.hour() * 60 + now.minute()) as u16,
            timezone: iana_time_zone::get_timezone().unwrap_or_else(|_| now.offset().to_string()),
        }
    }
    fn advance(&mut self) {
        let now = Instant::now();
        let wakes = platform::wake_count();
        if wakes == self.last_wake
            && !platform::asleep()
            && self
                .gateway
                .snapshot()
                .is_ok_and(|s| s.session.phase == Phase::Working)
        {
            self.elapsed_ms = self
                .elapsed_ms
                .saturating_add(now.duration_since(self.last_tick).as_millis() as u64)
                .min(awaytime_core::MAX_INTEGER);
        }
        self.last_wake = wakes;
        self.last_tick = now;
    }
    fn save(&mut self, mut snapshot: Snapshot) -> Result<(), String> {
        snapshot.session.work_elapsed_ms = self.elapsed_ms;
        self.store.save(&snapshot).map_err(|e| e.to_string())
    }
}
#[tauri::command]
fn proof_status(
    app: tauri::AppHandle,
    state: tauri::State<'_, Shared>,
) -> Result<ProofStatus, String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    let display_count = monitors.len();
    let signature: Vec<_> = monitors
        .iter()
        .map(|m| {
            (
                m.position().x,
                m.position().y,
                m.size().width,
                m.size().height,
            )
        })
        .collect();
    let mut runtime = state.lock().map_err(|_| "Proof runtime unavailable")?;
    runtime.advance();
    if runtime.preview_deadline.is_some() && runtime.preview_monitors != signature {
        let deadline = runtime.preview_deadline;
        let foreground = runtime.foreground;
        close_preview(&mut runtime);
        if !monitors.is_empty() {
            match show_preview(&app, &mut runtime) {
                Ok(()) => {
                    runtime.preview_deadline = deadline;
                    runtime.foreground = foreground;
                }
                Err(error) => runtime.last_error = Some(error),
            }
        }
    }
    let idle_ms = match platform::idle_ms() {
        Ok(ms) => {
            runtime
                .capabilities
                .insert("idle".into(), Capability::Supported);
            Some(ms)
        }
        Err(error) => {
            runtime
                .capabilities
                .insert("idle".into(), Capability::Unavailable(error));
            None
        }
    };
    let snapshot = runtime.gateway.snapshot()?;
    Ok(ProofStatus {
        phase: snapshot.session.phase,
        elapsed_ms: runtime.elapsed_ms,
        idle_ms,
        display_count,
        preview_active: runtime.preview_deadline.is_some(),
        preview_window_count: app
            .webview_windows()
            .keys()
            .filter(|label| label.starts_with("break-"))
            .count(),
        wake_count: platform::wake_count(),
        capabilities: runtime.capabilities.clone(),
        storage_notice: runtime.storage_notice.clone(),
        last_error: runtime.last_error.clone(),
    })
}
#[tauri::command]
fn dispatch(
    app: tauri::AppHandle,
    state: tauri::State<'_, Shared>,
    command: Command,
) -> Result<(), String> {
    dispatch_from(&app, &state, command, Source::Ui)
}
fn dispatch_from(
    app: &tauri::AppHandle,
    state: &Shared,
    command: Command,
    source: Source,
) -> Result<(), String> {
    let result: Result<(), String> = (|| {
        let mut runtime = state.lock().map_err(|_| "Proof runtime unavailable")?;
        runtime.advance();
        if runtime.storage_blocked
            && !matches!(command, Command::PreviewBreak | Command::EndPreview)
        {
            return Err("Saved data could not be loaded. Resolve the storage error before changing the session.".into());
        }
        let (snapshot, effects) = runtime
            .gateway
            .dispatch(command, source, &runtime.clock())?;
        for effect in effects {
            match effect {
                Effect::Persist => runtime.save(snapshot.clone())?,
                Effect::ShowPreview => show_preview(app, &mut runtime)?,
                Effect::ClosePreview => close_preview(&mut runtime),
            }
        }
        runtime.last_error = None;
        Ok(())
    })();
    if let Err(ref error) = result {
        if let Ok(mut runtime) = state.lock() {
            runtime.last_error = Some(error.clone());
        }
    }
    result
}
fn show_preview(app: &tauri::AppHandle, runtime: &mut ProofRuntime) -> Result<(), String> {
    if runtime.preview_deadline.is_some() {
        return Ok(());
    }
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    if monitors.is_empty() {
        return Err("No displays available".into());
    }
    runtime.foreground = platform::foreground();
    runtime.preview_generation += 1;
    let mut scope = ResourceScope::default();
    for (index, monitor) in monitors.iter().enumerate() {
        let label = format!("break-{}-{index}", runtime.preview_generation);
        let window = match WebviewWindowBuilder::new(
            app,
            &label,
            WebviewUrl::App("index.html?break=1".into()),
        )
        .title("AwayTime — Break preview")
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .visible_on_all_workspaces(true)
        .skip_taskbar(true)
        .visible(false)
        .build()
        {
            Ok(window) => window,
            Err(error) => {
                scope.close();
                platform::restore_foreground(runtime.foreground);
                return Err(error.to_string());
            }
        };
        let release = window.clone();
        scope.register(label, move || {
            let _ = release.close();
        });
        if let Err(error) = window
            .set_position(PhysicalPosition::new(
                monitor.position().x,
                monitor.position().y,
            ))
            .and_then(|_| {
                window.set_size(PhysicalSize::new(
                    monitor.size().width,
                    monitor.size().height,
                ))
            })
            .and_then(|_| window.show())
        {
            scope.close();
            platform::restore_foreground(runtime.foreground);
            return Err(error.to_string());
        }
    }
    runtime.preview_monitors = monitors
        .iter()
        .map(|m| {
            (
                m.position().x,
                m.position().y,
                m.size().width,
                m.size().height,
            )
        })
        .collect();
    runtime.preview = scope;
    runtime.preview_deadline = Some(Instant::now() + Duration::from_secs(15));
    Ok(())
}
fn close_preview(runtime: &mut ProofRuntime) {
    let was_active = runtime.preview_deadline.take().is_some();
    runtime.preview.close();
    runtime.preview_monitors.clear();
    if was_active {
        platform::restore_foreground(runtime.foreground);
    }
}
fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show_main(app)))
        .invoke_handler(tauri::generate_handler![proof_status, dispatch])
        .setup(|app| {
            let store = FileStore::new(app.path().app_data_dir()?);
            let (snapshot, storage_notice, storage_blocked) = match store.load() {
                Ok(Some(loaded)) => (loaded.snapshot, if loaded.recovered_backup { Some("Recovered the previous valid snapshot. The latest update may have been lost.".into()) } else {None}, false),
                Ok(None) => (Snapshot::default(), None, false),
                Err(error) => (Snapshot::default(), Some(format!("Saved data unavailable: {error}")), true),
            };
            let mut resources = ResourceScope::default();
            let mut capabilities = BTreeMap::new();
            for name in ["activity", "notifications", "audio", "focus", "calendar", "automation", "scripting", "global_shortcuts"] {
                capabilities.insert(name.into(), Capability::Unavailable("Not configured in Phase 0".into()));
            }
            capabilities.insert("storage".into(), if storage_blocked { Capability::Unavailable("Saved data unavailable".into()) } else {Capability::Supported});
            capabilities.insert("displays".into(), Capability::Supported);
            capabilities.insert("focus_restore".into(), if cfg!(target_os = "linux") { Capability::Unavailable("Linux focus restoration pending window-manager integration".into()) } else {Capability::Supported});
            match platform::watch_power() {
                Ok(release) => { resources.register("power".into(), release); capabilities.insert("sleep_wake".into(), Capability::Supported); }
                Err(error) => { capabilities.insert("sleep_wake".into(), Capability::Unavailable(error)); }
            }
            let elapsed_ms = snapshot.session.work_elapsed_ms;
            let runtime = Arc::new(Mutex::new(ProofRuntime { gateway: Gateway::new(snapshot).map_err(std::io::Error::other)?, store,
                origin: Instant::now(), last_tick: Instant::now(), elapsed_ms, last_wake: platform::wake_count(),
                preview_deadline: None, preview_generation: 0, preview_monitors: vec![], foreground: 0, preview: ResourceScope::default(), resources,
                capabilities, storage_notice, last_error: None, storage_blocked }));
            app.manage(runtime.clone());
            let items = [
                MenuItem::with_id(app, "show", "Open AwayTime", true, None::<&str>)?,
                MenuItem::with_id(app, "start", "Start session", true, None::<&str>)?,
                MenuItem::with_id(app, "pause", "Pause", true, None::<&str>)?,
                MenuItem::with_id(app, "resume", "Resume", true, None::<&str>)?,
                MenuItem::with_id(app, "stop", "Stop session", true, None::<&str>)?,
                MenuItem::with_id(app, "preview", "Preview a break", true, None::<&str>)?,
                MenuItem::with_id(app, "end", "End preview", true, None::<&str>)?,
                MenuItem::with_id(app, "quit", "Quit AwayTime", true, None::<&str>)?,
            ];
            let refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = items.iter().map(|item| item as _).collect();
            let menu = Menu::with_items(app, &refs)?;
            let mut tray = TrayIconBuilder::with_id("awaytime").tooltip("AwayTime").menu(&menu);
            if let Some(icon) = app.default_window_icon() { tray = tray.icon(icon.clone()); }
            tray.build(app)?.on_menu_event(|app, event| {
                let state = app.state::<Shared>();
                let command = match event.id.as_ref() {
                    "show" => {show_main(app); return;}, "quit" => {app.exit(0); return;},
                    "start" => Command::Start, "pause" => Command::Pause, "resume" => Command::Resume,
                    "stop" => Command::Stop, "preview" => Command::PreviewBreak, "end" => Command::EndPreview,
                    _ => return,
                };
                let _ = dispatch_from(app, &state, command, Source::Tray);
            });
            let stop = Arc::new(AtomicBool::new(false));
            let stop_owned = stop.clone();
            let handle = app.handle().clone();
            let worker_runtime = runtime.clone();
            let worker = std::thread::spawn(move || {
                while !stop.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(250));
                    if stop.load(Ordering::SeqCst) { break; }
                    let should_close = match worker_runtime.try_lock() {
                        Ok(mut state) => {
                            state.advance();
                            state.preview_deadline.is_some_and(|deadline| Instant::now() >= deadline || platform::asleep())
                        }
                        Err(std::sync::TryLockError::WouldBlock) => continue,
                        Err(std::sync::TryLockError::Poisoned(_)) => break,
                    };
                    if should_close {
                        let state = worker_runtime.clone();
                        let _ = handle.run_on_main_thread(move || { if let Ok(mut state) = state.lock() { close_preview(&mut state); } });
                    }
                }
            });
            runtime.lock().map_err(|_| std::io::Error::other("Runtime unavailable"))?.resources.register("poller".into(), move || {
                stop_owned.store(true, Ordering::SeqCst);
                let _ = worker.join();
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested {api, ..} = event { api.prevent_close(); let _ = window.hide(); }
            }
        });
    builder
        .build(tauri::generate_context!())
        .expect("Could not start AwayTime")
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = event {
                show_main(app);
            }
            if let tauri::RunEvent::Exit = event {
                if let Some(state) = app.try_state::<Shared>() {
                    if let Ok(mut runtime) = state.lock() {
                        runtime.advance();
                        if !runtime.storage_blocked {
                            if let Ok(snapshot) = runtime.gateway.snapshot() {
                                // Avoid creating storage on a quit with no session changes.
                                if snapshot != Snapshot::default() || runtime.elapsed_ms > 0 {
                                    if let Err(error) = runtime.save(snapshot) {
                                        eprintln!("Could not save session: {error}");
                                    }
                                }
                            }
                        }
                        close_preview(&mut runtime);
                        runtime.resources.close();
                    }
                }
            }
        });
}
