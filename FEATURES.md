# AwayTime — features and progress

Updated: 2026-10-13 (Asia/Kathmandu)

## Scope

Build AwayTime, a cross-platform desktop screen-break app for macOS, Windows, and Linux. iPhone/iPad synchronization, pairing, mobile app blocking, and Apple Watch mirroring are outside the project scope.

**Proposed stack, pending integration proof:** Tauri 2 + Rust + React/TypeScript + Vite, with versioned JSON settings and SQLite history later. Phase 0 is in progress: Rust foundation, React/TypeScript interface, and Tauri proof shell are implemented locally. Stack acceptance and cross-platform runtime verification are pending. Exact OS versions, architectures, Linux environments, and platform capabilities remain open under DEC-02. See [architecture](docs/ARCHITECTURE.md) and [support matrix](docs/SUPPORT_MATRIX.md).

This tracker defines AwayTime's feature requirements and acceptance checks. Checked items record implemented and verified behavior; unchecked items are planned. `[Design]` marks proposed engineering behavior.

## Progress rules

A feature checkbox means **implemented and verified**, including its relevant core, adapter, and UI tests. Unchecked means unfinished. Record partial progress as `doing` in the work log instead of checking it. Never mark a feature done merely because a test specification exists.

Each ID stays stable after edits. New items get new IDs; retired IDs remain in the decisions log. Priorities: **P0** usable break loop; **P1** daily-use features; **P2** advanced integration. Dependencies override priority: architecture and the core adapter come first.

Current verified progress: **3/165 feature items**, **11/50 core behavior cases**, verified against the Rust foundation on macOS arm64. Snapshot migrations/recovery, atomic writes, and local-first storage pass their foundation tests. Complete cross-platform native/UI verification remains pending. PH-00 remains open for project setup, integration proof, and support checks. Use `python3 run_contract_tests.py --progress` to recount feature and core-test checkboxes after updates.

Record implementation work and evidence here as development proceeds:

| Date | Item IDs | Status (todo/doing/blocked/done) | Evidence / next step |
| --- | --- | --- | --- |
| 2026-10-13 | CORE-01 | doing | Rust settings validation and pause-gate adapter pass 11/50 shared cases; remaining operations pending. |
| 2026-10-13 | AR-01–AR-07, AR-09, PL-05–PL-07, AT-08, AT-33 | done | Pure boundary, injected-time/state behavior, capability interfaces, JSON migration/recovery, concurrent writes, and interrupted-process commit tested locally; 15 Rust tests pass. |
| 2026-10-13 | AR-08, AR-11 | doing | Shared gateway/resource scope tested; app/tray commands and native quit implemented; shortcut/script entry points and complete native cleanup verification pending. |
| 2026-10-13 | DEC-01, DEC-02, PL-09, AT-37, PH-00 | doing | Tauri Mac development app built and partially exercised; three-platform CI configured. Real sleep/wake, complete display/lifecycle/resource checks, and Windows/Linux execution remain pending. |

### Preparation already completed

- [x] DOC-01 — Write feature inventory and progress rules.
- [x] TEST-01 — Write a language-neutral test contract and executable test harness.
- [x] DOC-02 — Organize features, architecture, and tests into dependency-ordered phases.

## Phase overview

Implemented Phase 0 foundation: **Rust core/runtime/persistence and JSON test adapter, React/TypeScript + Vite interface, and a Tauri desktop proof shell**. Local automated checks pass on macOS arm64. Shared test fixtures and platform capability limits remain the basis for cross-platform verification. Native/UI checks are recorded separately in the support matrix.

Planned stack: **Tauri 2 + Rust + React/TypeScript + Vite**, pending the [Phase 0 integration proof](docs/ARCHITECTURE.md#integration-proof-before-stack-acceptance). Use the fixtures and data contract as the behavior specification. Accept the proposed stack and settle platform support before committing to the full implementation or starting dependent phases; the full corpus and release checks remain later gates.

Each feature, architecture item, and test has one canonical checkbox below. The same test may need rerunning in later phases and on each target. Test IDs refer to the existing JSON fixtures or pending native scenarios. Feature IDs, priorities, and acceptance checks define the roadmap; verify each phase deliverable on its applicable desktop targets. Commercial work stays deferred, and mobile sync stays excluded.

### Platform-specific feature scope

Common break behavior and screens are intended to be shared. Operating-system services need separate implementations and verification. These scopes do not assert current capability support:

| Feature family | Cross-platform plan |
| --- | --- |
| Menu bar, locking, and break screens | Menu bar on macOS, tray/menu controls on Windows/Linux; verify lock actions, display coverage, focus, and wake per environment |
| Accessibility | VoiceOver on macOS; equivalent screen-reader and keyboard checks on Windows/Linux |
| Focus and activity detection | FO-01/FO-02 remain macOS Focus integration; detect meetings/video/recording only where verified and expose capability limits |
| Automation and scripting | AU-01 and the AppleScript part of AU-02 are macOS-specific; AU-03–AU-10 and SC-01–SC-07 use common commands with verified platform-specific entry points |
| Advanced lock-screen/screensaver status | FO-04/FO-05 only on verified OS/environment combinations |
| Distribution | Signing/installers/updates per target; notarization is macOS-specific, and PL-14 remains the Homebrew channel |

Every release platform must pass the common behavior and applicable native/UI checks. Unsupported controls follow FO-06; a missing required capability must be resolved or reflected in the agreed support scope, not silently treated as parity.

Phase feature counts exclude architecture/test/phase gates. A phase gate means all its applicable feature and test checks are complete and evidence is logged. Checkboxes are the source of truth; update summary counts when progress changes. Accessibility, permission handling, recovery, and testing begin with the relevant phase rather than waiting for release.

| Phase | Outcome | Feature items | Verified | Exit gate |
| --- | --- | ---: | ---: | --- |
| 0 | [Foundation and portable architecture](#phase-0--foundation-and-portable-architecture) | 4 | 3/4 | PH-00 |
| 1 | [Break engine and discipline rules](#phase-1--break-engine-and-discipline-rules) | 20 | 0/20 | PH-01 |
| 2 | [First usable cross-platform desktop app](#phase-2--first-usable-cross-platform-desktop-app) | 35 | 0/35 | PH-02 |
| 3 | [Idle recovery and break scheduling](#phase-3--idle-recovery-and-break-scheduling) | 27 | 0/27 | PH-03 |
| 4 | [Activity-aware pausing and OS context](#phase-4--activity-aware-pausing-and-os-context) | 22 | 0/22 | PH-04 |
| 5 | [Wellness, personalization, and languages](#phase-5--wellness-personalization-and-languages) | 18 | 0/18 | PH-05 |
| 6 | [Statistics and data controls](#phase-6--statistics-and-data-controls) | 16 | 0/16 | PH-06 |
| 7 | [Automation and advanced platform integrations](#phase-7--automation-and-advanced-platform-integrations) | 20 | 0/20 | PH-07 |
| 8 | [Cross-platform verification and release](#phase-8--cross-platform-verification-and-release) | 3 | 0/3 | PH-08 |

## Phase 0 — Foundation and portable architecture

Set up the project structure, time model, persistence boundaries, and test interface. Prove required desktop integrations before accepting the proposed stack.

**Depends on:** none; start here.

**Preparation:** Documentation, data contract, language-neutral fixtures, and Python test harness.

**Next deliverable:** A small Tauri proof on macOS, Windows, and selected Linux environments covering tray controls, two-monitor break screens, idle detection, and sleep/wake recovery. Run a Rust adapter against the 11 settings/pause-gate fixtures and record resource measurements and platform limits. Resolve DEC-01/DEC-02 before committing to the full implementation; if accepted, build the foundation according to the data and behavioral contracts. Proof details are in [architecture](docs/ARCHITECTURE.md#integration-proof-before-stack-acceptance). The proof is in progress on macOS; native Windows/Linux execution and full display/sleep/resource evidence remain pending.

### Architecture checklist

- [x] AR-01 — Core contains no UI framework, platform API, or browser imports, real clock reads, filesystem access, or OS permission requests.
- [x] AR-02 — Represent transitions as explicit states and commands; return effects as data rather than executing them inside the core.
- [x] AR-03 — Inject elapsed time and local calendar time separately; elapsed timers use a monotonic clock (one that wall-clock changes cannot move backward).
- [x] AR-04 — Define replaceable adapters for activity, idle, displays, notifications, audio, persistence, Focus, calendar, automation, and scripting.
- [x] AR-05 — Keep platform capabilities explicit: supported, unsupported, or permission required. No fake cross-platform parity.
- [x] AR-06 — Use versioned, language-neutral settings/events with integer milliseconds and local schedule minutes; specify rounding and overflow behavior.
- [x] AR-07 — Keep data migration independent of UI; export/import round-trips preserve custom settings.
- [ ] AR-08 — Every UI, shortcut, and scripted action calls the same core command.
- [x] AR-09 — Reuse the JSON contract cases on every port; adapter translates public test operations to native core APIs.
- [ ] AR-11 — Cancel/release OS subscriptions, windows, audio, and pending automation tasks on stop/quit.

### Setup, storage, privacy, and distribution (P1)

- [x] PL-05 — Versioned settings migrations and corruption recovery. [Design]
- [x] PL-06 — Atomic local writes and durable session restore. [Design]
- [x] PL-07 — Local-first storage; no account required for core reminders. [Design]
- [ ] PL-09 — Minimum supported OS matrix and hardware verification. [Design]

### Integration and reliability tests to implement and pass

These scenarios require implementation; they are not part of the initial executable JSON cases.

- [x] AT-08 — Save/load preserves interval, settings, quota, and cycle; corrupt/old state recovers without losing unrelated data.
- [x] AT-33 — Interrupted atomic write preserves last valid state; migrations tolerate unknown optional fields and reject incompatible future versions clearly.
- [ ] AT-37 — Continuous integration fails on behavioral failures, missing adapters, fixture errors, and regression tests; archive results per revision.
- [ ] AT-42 — Windows/Linux runtime proof remains open until native execution on both platforms verifies tray commands, background window lifecycle, two-display previews, idle observations, and sleep/wake recovery. Record OS/environment, hardware, results, and capability limits in the support matrix; compilation or Mac-only evidence does not pass this test.

### Phase completion

- [ ] PH-00 — Core builds independently of UI/OS frameworks; fixture validation runs in automated checks; persistence recovery tests pass for the accepted implementation. Complete the integration proof, including the open Windows/Linux runtime test AT-42, and resolve DEC-01/DEC-02 first; settle time/event schema before dependent phases.

## Phase 1 — Break engine and discipline rules

Make break behavior predictable and testable without launching a desktop interface.

**Depends on:** Phase 0; rerun earlier checks affected by new adapters or UI.

**Deliverable:** Configurable short/long/custom breaks, start/stop/pause/resume/reset, skip modes, snooze quotas, and independent pause reasons.

### Break engine (P0)

- [ ] BR-01 — Configurable work interval; example: 20 minutes.
- [ ] BR-02 — Configurable short-break duration; example: 20 seconds.
- [ ] BR-03 — Optional long breaks with configurable duration and frequency.
- [ ] BR-04 — Short, long, and custom breaks started on demand.
- [ ] BR-05 — Start, stop, pause, temporary pause, resume, and reset session/cycle.
- [ ] BR-06 — Resume saved session after relaunch without counting downtime as work. [Design]
- [ ] BR-07 — One active break at a time; deterministic transitions. [Design]
- [ ] BR-08 — Persist settings and session state with versioned data. [Design]
- [ ] BR-09 — Apply changed settings without restarting. [Design]
- [ ] BR-10 — Reject invalid, negative, non-finite, or overflowing durations. [Design]

### Skip, snooze, and enforcement (P0)

- [ ] DS-01 — Casual mode: immediate skip.
- [ ] DS-02 — Balanced mode: delayed skip.
- [ ] DS-03 — Hardcore mode: disable skip before and during regular breaks.
- [ ] DS-04 — Allow optional early end after sufficient rest.
- [ ] DS-06 — Per-session and per-day snooze limits.
- [ ] DS-07 — Advance skip with daily limit.
- [ ] DS-08 — Manual completion.
- [ ] DS-10 — Reset daily limits at local midnight, session limits at new session. [Design]
- [ ] DS-11 — Keep full break duration after snooze. [Design]

### Smart Pause (P1)

- [ ] SP-14 — Track multiple simultaneous pause reasons; clear only the one that ended. [Design]

### Portable behavior tests to pass

- [x] T-001 — Valid 20-minute/20-second settings accepted. Covers BR-01, BR-02.
- [x] T-002 — Zero work duration rejected. Covers BR-10.
- [x] T-003 — Negative break duration rejected. Covers BR-10.
- [x] T-004 — Boolean work duration rejected. Covers BR-10.
- [x] T-005 — Fractional milliseconds rejected. Covers BR-10.
- [x] T-006 — Negative snooze limit rejected. Covers BR-10.
- [x] T-007 — Enabled long break requires a frequency. Covers BR-03, BR-10.
- [ ] T-008 — Before threshold no break starts. Covers BR-01, BR-02, BR-07.
- [ ] T-009 — Exact threshold starts one short break. Covers BR-01, BR-02, BR-07.
- [ ] T-010 — Repeated tick at threshold cannot duplicate a break. Covers BR-01, BR-02, BR-07.
- [ ] T-011 — Completed rest returns to fresh work session. Covers BR-01, BR-02, BR-07.
- [ ] T-012 — Paused time does not count as work. Covers ID-02, BR-05.
- [ ] T-013 — Stop prevents a future overdue break. Covers BR-05, OH-06.
- [ ] T-014 — Snooze at threshold wins over starting break. Covers RM-04, BR-07.
- [ ] T-015 — Snoozed break retains full rest duration. Covers DS-11.
- [ ] T-016 — Skip does not count as completed rest. Covers DS-01, ST-01.
- [ ] T-017 — On-demand custom break has its requested duration. Covers BR-04.
- [ ] T-018 — Long break follows configured completed-break cycle. Covers BR-03.
- [ ] T-019 — Boundary pause prevents short-break start. Covers SP-14, BR-07.
- [ ] T-020 — One late tick crosses work and rest exactly once. Covers BR-07.
- [ ] T-021 — Completing rest and stopping at the same instant leaves stopped state. Covers BR-07.
- [x] T-022 — No pause reasons permits break. Covers SP-14.
- [x] T-023 — Two pause reasons remain independent and deduplicated. Covers SP-14.
- [x] T-024 — Excluding one reason leaves the other active. Covers SP-04, SP-14.
- [x] T-025 — Excluded activity cannot block a break. Covers SP-01.
- [ ] T-037 — Casual mode permits immediate skip. Covers DS-01, DS-02, DS-03.
- [ ] T-038 — Balanced mode blocks skip before delay. Covers DS-01, DS-02, DS-03.
- [ ] T-039 — Balanced mode permits skip at exact delay. Covers DS-01, DS-02, DS-03.
- [ ] T-040 — Hardcore blocks ordinary skip. Covers DS-01, DS-02, DS-03.
- [ ] T-042 — Unlimited snooze has no accidental cap. Covers DS-06.
- [ ] T-043 — Session limit blocks snooze at boundary. Covers DS-06.
- [ ] T-044 — Daily limit blocks snooze at boundary. Covers DS-06.
- [ ] T-045 — Zero snooze limit blocks all snoozes. Covers DS-06.
- [ ] T-046 — Snooze permitted while both limits remain. Covers DS-06.

### Integration and reliability tests to implement and pass

These scenarios require implementation; they are not part of the initial executable JSON cases.

- [ ] AT-09 — Duplicate and out-of-order events cannot cause negative time, repeated transitions, or restored canceled effects.
- [ ] AT-32 — Seeded random command sequences preserve invariants: one active break, nonnegative remaining time, no rest counted twice, and stopped stays stopped until explicit start.

### Phase completion

- [ ] PH-01 — All phase behavior tests pass against the real core; randomized event tests preserve invariants. Specify early-end and cycle policies from DEC-03/DEC-04. Later phases verify these rules through real OS adapters.

## Phase 2 — First usable cross-platform desktop app

Deliver a complete daily-use loop: configure a timer, work, receive notice, take a break, and return to work.

**Depends on:** Phase 1; rerun earlier checks affected by new adapters or UI.

**Deliverable:** Shared settings and break screens, native menu bar/tray controls, advance reminders, keyboard controls, sound, supported lock actions, and accessible baseline controls on the agreed desktop matrix. Use Tauri/React only after Phase 0 accepts the proposal; implement and verify native services separately per platform.

### Reminders and nudges (P0)

- [ ] RM-01 — Enable/disable advance reminders; configure lead time and visible duration.
- [ ] RM-02 — Show time until break and time spent working.
- [ ] RM-03 — Start break now from its reminder.
- [ ] RM-04 — Snooze by 1, 5, or 15 minutes.
- [ ] RM-05 — Floating cursor countdown with upcoming break name.
- [ ] RM-07 — Dismissible notification and compact presentation.
- [ ] RM-08 — No duplicate reminder or completion sound. [Design]
- [ ] RM-09 — Notification positioning.
- [ ] RM-10 — Configurable countdown length.
- [ ] RM-11 — Snooze-count alert toggle.

### Menu bar and keyboard (P0)

- [ ] MB-01 — Live state and quick access to settings.
- [ ] MB-02 — Countdown, time since rest, pause reason, and state indicators.
- [ ] MB-04 — Right-click snooze actions.
- [ ] MB-05 — Icon visibility and text styles.
- [ ] MB-06 — Global shortcuts for panel, start/stop, pause/resume, snooze, and breaks.
- [ ] MB-07 — Conflict detection and editable shortcuts. [Design]
- [ ] MB-08 — Keyboard controls do not trigger typing deferral. [Design]

### Break screens and customization (P1)

- [ ] UX-01 — Show full-screen break UI across multiple displays.
- [ ] UX-02 — Remaining time and end-of-break sound.
- [ ] UX-08 — Restore previous app focus after break. [Design]
- [ ] UX-09 — Handle display rotation, disconnect, clamshell mode, and wake mid-break. [Design]
- [ ] UX-10 — Respect contrast, motion, and transparency preferences. [Design]
- [ ] UX-11 — Start chime toggle.
- [ ] UX-13 — Manual lock button.

### Break engine (P0)

- [ ] BR-11 — Long-break sleep prevention.

### Skip, snooze, and enforcement (P0)

- [ ] DS-05 — Automatic desktop locking at break start on supported platforms.
- [ ] DS-09 — Configurable double-Escape action.

### Setup, storage, privacy, and distribution (P1)

- [ ] PL-01 — First-run explanation and timer configuration. [Design]
- [ ] PL-04 — Start-at-login option; reliable stop/quit behavior. [Design]

### Accessibility and localization (P1)

- [ ] AC-01 — Screen-reader labels and announcements; VoiceOver on macOS and equivalent checks on Windows/Linux.
- [ ] AC-02 — Keyboard navigation.
- [ ] AC-03 — Reduced Motion, transparency, contrast.
- [ ] AC-04 — Non-color state indicators.
- [ ] AC-09 — Test large text, long translations, and focus order. [Design]

### OS integrations (P2)

- [ ] FO-06 — Hide unsupported controls according to OS capability. [Design]

### Integration and reliability tests to implement and pass

These scenarios require implementation; they are not part of the initial executable JSON cases.

- [ ] AT-10 — Reminder lead-time boundary, snooze during visible countdown, and exact due-time race produce one correct reminder.
- [ ] AT-16 — Rotation, clamshell, Spaces, fullscreen, display disconnect, and wake maintain break coverage and focus restoration.
- [ ] AT-17 — Notification dismissal, shortcut conflicts, menu actions, and global shortcuts dispatch the same command once.
- [ ] AT-23 — Start-at-login, quit, reinstall/update, and single-instance behavior preserve settings without surprise startup.
- [ ] AT-40 — Manual lock and long-break sleep prevention follow user settings; cleanup removes keep-awake assertions on stop/quit.

### Phase completion

- [ ] PH-02 — Complete the work → reminder → break → work flow on every agreed desktop target, with multi-display, stop/quit, keyboard, and platform screen-reader checks. Finish baseline accessibility here; AT-22 adds translated-layout verification in Phase 5.

## Phase 3 — Idle recovery and break scheduling

Respect natural rest and work schedules, including overnight and fixed-time breaks.

**Depends on:** Phase 2; rerun earlier checks affected by new adapters or UI.

**Deliverable:** Idle/lock/sleep recovery, Office Hours, planned-break editing, rest credit, overlap handling, and daily quota boundaries.

### Idle time and natural rest (P0)

- [ ] ID-01 — Detect idle and locked states; pause or reset according to rest taken.
- [ ] ID-02 — Do not count sleep, idle, or locked time as active screen use. [Design]
- [ ] ID-03 — Do not reset a session after a brief lock/unlock. [Design]
- [ ] ID-04 — Natural-rest decision with undo.
- [ ] ID-05 — Last-away correction control.
- [ ] ID-06 — Keep-awake interference warning.
- [ ] ID-07 — Normalize late/out-of-order sensor events without negative timers. [Design]
- [ ] ID-08 — Configurable pause/reset thresholds.

### Office Hours (P1)

- [ ] OH-01 — Enable/disable scheduled regular reminders.
- [ ] OH-02 — Choose active weekdays and start/end times.
- [ ] OH-03 — Per-day/overnight schedules.
- [ ] OH-04 — Keep local wall-clock schedule when timezone changes. [Design]
- [ ] OH-05 — Recompute next activation after sleep or clock changes. [Design]
- [ ] OH-06 — Manual stop remains stopped until explicit resume. [Design]

### Planned breaks (P1)

- [ ] PB-01 — Create, edit, enable/disable, and delete recurring planned breaks.
- [ ] PB-02 — Name, icon, start time, duration, and selected repeat days.
- [ ] PB-03 — Duration from one minute to two hours.
- [ ] PB-04 — Weekday/weekend/every-day presets; require at least one day.
- [ ] PB-05 — Reject overlaps, including intervals crossing midnight.
- [ ] PB-06 — Operate independently of Office Hours while app runs.
- [ ] PB-07 — Suppress nearby regular breaks; never stack two breaks.
- [ ] PB-08 — Defer for activity, Focus, or manual pause, then apply grace period.
- [ ] PB-09 — Count idle/locked time within planned window toward rest.
- [ ] PB-10 — Start now, snooze five minutes, or skip today without discipline penalty.
- [ ] PB-11 — Ignore an occurrence when too little useful time remains.
- [ ] PB-12 — Track planned breaks separately from regular-break streaks.
- [ ] PB-13 — Do not retroactively restart stopped app. [Design]

### Portable behavior tests to pass

- [ ] T-026 — Office Hours includes opening boundary. Covers OH-02.
- [ ] T-027 — Office Hours excludes closing boundary. Covers OH-02.
- [ ] T-028 — Inactive weekday stays inactive. Covers OH-02.
- [ ] T-029 — Monday overnight hours include Tuesday 01:00. Covers OH-03.
- [ ] T-030 — Sunday overnight hours wrap into Monday. Covers OH-03.
- [ ] T-031 — Touching planned breaks do not overlap. Covers PB-05.
- [ ] T-032 — Overlapping planned breaks rejected. Covers PB-05.
- [ ] T-033 — Sunday/Monday overlap detected across midnight. Covers PB-05.
- [ ] T-034 — Overlapping away samples cannot double-count rest. Covers PB-09.
- [ ] T-035 — Full away window completes planned rest. Covers PB-09.
- [ ] T-036 — Away time outside planned window earns no rest. Covers PB-09.
- [ ] T-041 — Planned skip-today bypasses hardcore restriction. Covers PB-10.

### Integration and reliability tests to implement and pass

These scenarios require implementation; they are not part of the initial executable JSON cases.

- [ ] AT-01 — Given work in progress, move wall clock backward/forward; elapsed work remains unchanged and next schedule recalculates.
- [ ] AT-02 — Spring-forward nonexistent local schedule time runs at first valid instant; fall-back repeated time runs once per occurrence ID. These are proposed policies to implement and document.
- [ ] AT-03 — Sleep/wake, brief lock/unlock, and long idle grant exactly the correct rest; no repeated lock or duplicate start effects.
- [ ] AT-04 — Midnight resets daily quotas and cycles once; timezone travel cannot reset quota twice for the same local-date ledger entry.
- [ ] AT-05 — Planned/regular collision yields one break; a missed planned occurrence is ignored without regular-skip penalty.
- [ ] AT-06 — Deferred planned break credits clipped idle time; too-short remainder follows a documented minimum-useful-duration policy.
- [ ] AT-07 — Manual stop survives scheduled windows, unlock, relaunch, and planned-break completion.

### Phase completion

- [ ] PH-03 — Phase behavior tests and scheduling/recovery tests pass, including midnight, timezone travel, daylight-saving changes, and planned/regular collisions. Finalize relevant thresholds and windows in DEC-03/DEC-04. Phase 4 adds live activity deferral.

## Phase 4 — Activity-aware pausing and OS context

Defer breaks during calls, recordings, focused activity, and configured calendar or Focus sessions.

**Depends on:** Phase 3; rerun earlier checks affected by new adapters or UI.

**Deliverable:** Camera/microphone, video, fullscreen, recording, typing/dragging/dictation, calendar, and macOS Focus adapters with configurable exclusions where supported. Verify each integration per platform and show capability/permission limits; shared UI does not imply shared sensor support.

### Smart Pause (P1)

- [ ] SP-01 — Separate switches for meetings, video, focus apps, games, recording/sharing.
- [ ] SP-02 — Meeting detection using camera or microphone activity.
- [ ] SP-03 — Include/exclude specific microphones and cameras, including external devices.
- [ ] SP-04 — Exclude specific apps from meeting detection.
- [ ] SP-05 — Video detection for frontmost only or background playback too.
- [ ] SP-06 — Video app exclusion list.
- [ ] SP-07 — Custom focus-app list: foreground, foreground/fullscreen, or merely open.
- [ ] SP-08 — Automatic fullscreen-game detection.
- [ ] SP-09 — Screen sharing/recording detection; suppress disruptive break UI.
- [ ] SP-10 — Recording app exclusions.
- [ ] SP-11 — Typing/dragging deferral.
- [ ] SP-12 — Dictation app selection and deferral.
- [ ] SP-13 — Configurable cooldown after activity ends.
- [ ] SP-15 — Show pause reason; avoid sticky typing/dragging or disconnected-device states. [Design]
- [ ] SP-16 — Meeting-time accounting toggle.
- [ ] SP-17 — Optional meeting alerts/ignore-once.

### OS integrations (P2)

- [ ] FO-01 — Configure macOS Focus filter to pause reminders.
- [ ] FO-02 — Resume safely after Focus ends.
- [ ] FO-03 — Calendar selection, accepted-only, private labels.

### Reminders and nudges (P0)

- [ ] RM-06 — Optional overtime nudge with pause suppression.

### Setup, storage, privacy, and distribution (P1)

- [ ] PL-02 — Request permissions only when dependent features are enabled. [Design]
- [ ] PL-03 — Permission denied/revoked recovery and clear capability status. [Design]

### Integration and reliability tests to implement and pass

These scenarios require implementation; they are not part of the initial executable JSON cases.

- [ ] AT-11 — Deny/revoke/regrant each permission; dependent features degrade clearly and resume after recovery.
- [ ] AT-12 — Connect/disconnect camera, microphone, and displays repeatedly; no stale pause reason, crash, or growing subscriptions.
- [ ] AT-13 — Each detector verifies active, excluded, background, fullscreen, and disabled states against supported apps.
- [ ] AT-14 — Calendar back-to-back/declined/private events and Focus toggles obey configured policy; clearing one does not clear another.
- [ ] AT-15 — Typing, dragging, dictation, and application shortcuts cannot leave a stuck pause after input ends.
- [ ] AT-38 — Test configurable idle thresholds, meeting-time accounting, ignore-once, notification positioning/countdown, and remaining-snooze alerts independently.

### Phase completion

- [ ] PH-04 — Permission-denied recovery, device churn, simultaneous pause reasons, cooldown, and detector exclusions pass native tests. Rerun Phase 1 pause-gate and Phase 3 planned-deferral cases with real sensor inputs.

## Phase 5 — Wellness, personalization, and languages

Add blink/posture nudges and customizable break experiences without sacrificing accessibility.

**Depends on:** Phase 4; rerun earlier checks affected by new adapters or UI.

**Deliverable:** Wellness reminders, display routing, backgrounds, imported sounds, messages, language selection, and localized UI.

### Blink and posture reminders (P1)

- [ ] WB-01 — Independent blink and posture reminder switches.
- [ ] WB-02 — Configurable frequency, presentation, and sound. [Design]
- [ ] WB-03 — Size/position, dimming, and volume.
- [ ] WB-04 — Every/active/specific display selection.
- [ ] WB-05 — Optional timer reset after breaks.
- [ ] WB-06 — Optional display during pauses.
- [ ] WB-07 — Recording invisibility and muting.
- [ ] WB-08 — Keep reminder windows correctly placed across display changes and Spaces. [Design]

### Break screens and customization (P1)

- [ ] UX-03 — Built-in and imported sounds; independent volume/mute. [Design]
- [ ] UX-04 — Custom image, system wallpaper, gradient, and animated backgrounds.
- [ ] UX-05 — Custom messages in any language.
- [ ] UX-06 — Multiline text; independent title/subtitle visibility.
- [ ] UX-07 — Optional retro reveal animation.
- [ ] UX-12 — Random short/long message lists.

### Accessibility and localization (P1)

- [ ] AC-05 — Language selector.
- [ ] AC-06 — Localized onboarding, settings, menus, notifications.
- [ ] AC-07 — Locale-aware durations, dates, schedules.
- [ ] AC-08 — English, German, Chinese (simplified/traditional), French, Japanese, Russian, Ukrainian.

### Integration and reliability tests to implement and pass

These scenarios require implementation; they are not part of the initial executable JSON cases.

- [ ] AT-18 — Posture/blink frequencies, resets, pause suppression, sound volumes, and display routing work independently.
- [ ] AT-19 — Screen recording hides/removes configured overlays and mutes corresponding reminder audio.
- [ ] AT-21 — Imported sound/image failures use a readable fallback; no blocking file/network work on UI thread.
- [ ] AT-22 — Platform screen readers, keyboard navigation, reduced motion, transparency, contrast, and translated layouts verified on device.
- [ ] AT-39 — With seeded random choice, short/long messages come only from their matching lists; empty lists use fallback; multiline text remains readable.

### Phase completion

- [ ] PH-05 — Reminder cadence/routing, recording privacy, import fallbacks, translated layouts, and accessibility tests pass. Rerun Phase 2 accessibility checks after each visual change.

## Phase 6 — Statistics and data controls

Explain screen habits with accurate measurements and user-controlled local data.

**Depends on:** Phase 5; rerun earlier checks affected by new adapters or UI.

**Deliverable:** Usage history in SQLite, daily summaries, app/site breakdowns, Screen Score, Quick Look, retention/export/deletion, and redacted diagnostics.

### Statistics (P1)

- [ ] ST-01 — Daily active screen time, break count, longest and median session.
- [ ] ST-02 — App usage during active sessions.
- [ ] ST-03 — Website usage by domain, enabled separately for supported browsers.
- [ ] ST-04 — Supported Safari-based and Chromium-based browser adapters.
- [ ] ST-05 — Firefox/Gecko adapter with permission support; App Store restrictions documented.
- [ ] ST-06 — Unsupported browser fallback to app totals.
- [ ] ST-07 — Combined app/site view.
- [ ] ST-08 — Screen Score from 0 to 100; session portion 60, adherence portion 40.
- [ ] ST-09 — Define and document AwayTime's Screen Score formula. [Design]
- [ ] ST-10 — Skip forgiveness.
- [ ] ST-11 — Prevent double-counting browser/app/domain time. [Design]
- [ ] ST-12 — Empty-day, local-date boundaries, rounding, and invalid-sample handling. [Design]
- [ ] ST-13 — Local data retention, export, and deletion controls. [Design]

### Menu bar and keyboard (P0)

- [ ] MB-03 — Quick Look session/score panel.

### Setup, storage, privacy, and distribution (P1)

- [ ] PL-08 — Explicit opt-in diagnostics; redact browser URLs, messages, and calendar names. [Design]
- [ ] PL-12 — Diagnostic export.

### Portable behavior tests to pass

- [ ] T-047 — Empty statistics contain zeros, not NaN. Covers ST-01, ST-12.
- [ ] T-048 — Odd session median uses middle sorted value. Covers ST-01, ST-12.
- [ ] T-049 — Even session median averages middle values. Covers ST-01, ST-12.
- [ ] T-050 — Website breakdown does not add to active total. Covers ST-11.

### Integration and reliability tests to implement and pass

These scenarios require implementation; they are not part of the initial executable JSON cases.

- [ ] AT-24 — Browser opt-out/unsupported/browser timeout keeps app totals and does not freeze UI or collect full URLs.
- [ ] AT-25 — Score stays within 0–100, is deterministic, and follows our published formula; qualifying rest forgives a skip once.
- [ ] AT-26 — Planned, skipped, completed, and early-ended breaks update counts according to documented semantics.
- [ ] AT-27 — App/domain totals reconcile within documented rounding tolerance; overlapping samples are not counted twice.
- [ ] AT-28 — Export/import and schema migration work across stacks; deletion removes local records and pending writes.

### Phase completion

- [ ] PH-06 — All 50 portable behavior cases now pass, plus browser consent/timeouts, score, reconciliation, and data-migration tests. Publish our score formula and retention policy under DEC-03/DEC-05.

## Phase 7 — Automation and advanced platform integrations

Expose reliable user-controlled actions and finish advanced desktop integrations.

**Depends on:** Phase 6; rerun earlier checks affected by new adapters or UI.

**Deliverable:** Break-triggered scripts, macOS Apple Shortcuts/AppleScript integrations, platform-specific external command entry points, supported lock-screen/screensaver status, and shared help/about/feedback controls. All entry points route through the same engine commands.

### Automations (P2)

- [ ] AU-01 — Run Apple Shortcuts at break start or end.
- [ ] AU-02 — Run AppleScript or shell scripts at break start or end.
- [ ] AU-03 — Add/edit/delete and enable/disable individual automations.
- [ ] AU-04 — Break-type selection.
- [ ] AU-05 — Example: pause/resume music or change screen brightness.
- [ ] AU-06 — Example: set Do Not Disturb or lock Mac.
- [ ] AU-07 — Example: user-configured Slack status script; no bundled account credentials.
- [ ] AU-08 — Timeout, error reporting, and cancellation without freezing UI. [Design]
- [ ] AU-09 — Stable break IDs prevent duplicate side effects. [Design]
- [ ] AU-10 — Explicitly enabled scripts only; no secrets in logs or exports. [Design]

### Scripting (P2)

- [ ] SC-01 — Start next break, long break, or custom-duration break.
- [ ] SC-02 — Custom message for one scripted break.
- [ ] SC-03 — Pause, timed pause, resume, and postpone.
- [ ] SC-04 — Restart/reset timers and cycle; open settings.
- [ ] SC-05 — Use commands through Shortcuts, Alfred, Raycast, or other scripts.
- [ ] SC-06 — Reject invalid commands and return actionable errors. [Design]
- [ ] SC-07 — Route scripting and UI actions through identical core commands. [Design]

### OS integrations (P2)

- [ ] FO-04 — Lock-screen countdown/placement preview.
- [ ] FO-05 — Optional screensaver status.

### Setup, storage, privacy, and distribution (P1)

- [ ] PL-13 — About/help/feedback controls.

### Integration and reliability tests to implement and pass

These scenarios require implementation; they are not part of the initial executable JSON cases.

- [ ] AT-20 — Auto-lock and lock-screen/screensaver status function only on supported OS versions; unlock does not restart rest.
- [ ] AT-29 — Automation timeout/failure/cancellation leaves UI responsive; duplicate events do not repeat successful actions.
- [ ] AT-30 — Disabled automation never executes; selection filters apply; script logs redact secrets and private data.
- [ ] AT-31 — Scripted/UI/keyboard commands match; malformed duration/message produces an error, not crash or code execution.

### Phase completion

- [ ] PH-07 — Automation timeout/cancellation/duplicate-effect tests pass; script, UI, and keyboard commands agree; supported lock-screen behavior is verified on device. Rerun Phase 2 auto-lock checks with advanced status enabled.

## Phase 8 — Cross-platform verification and release

Complete cross-platform behavior and native verification, measure reliability, and package releases. Stack selection and the small integration proof happen in Phase 0, not at release time.

**Depends on:** Phase 7; rerun earlier checks affected by new adapters or UI.

**Deliverable:** Full shared-corpus verification of the accepted implementation on every agreed desktop target, measured performance/soak results per target, platform-appropriate installers/signing, macOS notarization, update policy, and Homebrew distribution for the Mac app.

### Architecture checklist

- [ ] AR-10 — Run the accepted implementation's test adapter against the same full corpus on every agreed desktop target before claiming portability is demonstrated; the Phase 0 subset proof is insufficient.

### Setup, storage, privacy, and distribution (P1)

- [ ] PL-10 — Platform-appropriate installers/signing, macOS notarization, and reproducible release builds. [Design]
- [ ] PL-11 — Update check, rollback strategy, and release notes. [Design]
- [ ] PL-14 — Homebrew installation channel.

### Integration and reliability tests to implement and pass

These scenarios require implementation; they are not part of the initial executable JSON cases.

- [ ] AT-34 — Instrument idle CPU, memory, battery, detector churn, and UI latency on representative hardware; set measured budgets before release.
- [ ] AT-35 — Long-running soak with thousands of transitions and device changes shows no continuing resource growth.
- [ ] AT-36 — Run the same full fixture corpus through the core adapter on every agreed desktop target; compare canonical outputs and explain native capability differences.
- [ ] AT-41 — Diagnostic export is redacted and explicit; help/about actions work without a configured email client; Homebrew install launches the signed version.

### Phase completion

- [ ] PH-08 — The accepted implementation passes the full shared corpus on every agreed desktop target, all applicable native/UI/regression tests pass on each agreed desktop target, soak/resource budgets are met, and each installed release is verified. Set measured budgets and distribution policy in DEC-05. Publishing follows a separate release action.

## Optional commercial features — deferred

Payment, licensing services, trials, and subscription distribution are optional future work. These items are deferred until the core app works; they are not first-release commitments.

- [ ] CM-01 — License activation/deactivation and device transfer.
- [ ] CM-02 — Seat additions and remote device reset.
- [ ] CM-03 — Update entitlement and optional renewal.
- [ ] CM-04 — Direct, App Store, and Setapp release channels.
- [ ] CM-05 — Stable/beta update channel.
- [ ] CM-06 — Optional trial.

## Portability approach

Keep decisions in plain data and portable functions. The proposed Rust engine owns shared behavior independently of React screens and Tauri. Platform modules supply OS information and execute effects. For example, a verified microphone adapter emits `meeting=true`; the core decides to defer a break. A Windows adapter can emit the same event without rewriting that decision. Unsupported detectors remain explicit capabilities.

Follow the JSON fixtures and [data contract](docs/DATA_CONTRACT.md). If the Phase 0 proof succeeds, implement the specified behavior in Rust and verify timing, persistence, migration, and cleanup. The application must compute results from core rules and pass the fixtures on every agreed desktop target.

## Executable behavior tests

Files alongside this document:

- `contract_cases.json` — Input/output examples independent of implementation language.
- `run_contract_tests.py` — Standard-library Python runner, including tests of the runner itself.

These are **test-first specifications for AwayTime**. The Rust adapter passes **11/50** behavior cases: settings validation and pause-gate decisions. The remaining 39 cases are pending. The runner fails without an adapter, and unimplemented adapter operations fail explicitly. Use --operation to select the implemented Phase 0 subset.

### Adapter contract, version 1

The runner starts the supplied command once per case. It writes one JSON object to standard input and expects one JSON object on standard output, followed by process exit. Diagnostic logs go to standard error. No shell interpolation is used. A nonzero exit, timeout, malformed JSON, missing field, wrong type, or mismatched value fails the case. Extra output fields are permitted.

Request: `{"version": 1, "operation": "...", "input": {...}}`.
Response: the operation's expected fields in the fixture, for example `{"valid": false, "errors": ["work_ms"]}`. Arrays are exact and ordered. Expected objects are recursively compared as subsets, so implementations may add diagnostic fields without breaking ports.

Public test operations:

| Operation | Rules for the test adapter |
| --- | --- |
| `validate_settings` | Positive integer work/short-break durations; nonnegative integer snooze limit; optional long break requires positive duration and positive frequency. Return `valid` and sorted invalid field names. Reject booleans as durations. |
| `interval_session` | Begin working at elapsed time zero. Events arrive in chronological order with `at_ms`; phases are `working`, `paused`, `break`, or `stopped`. Work accrues only while working; rest only during break. A completed break resets work, advances cycle, and returns to working. At a break threshold, process the event first, then decide whether to start a break. At a completed break boundary, complete the break before processing the event. `pause`/`resume` freeze/unfreeze work; `stop` suppresses later ticks; `snooze` adds one full specified interval to the work target; `skip` resets work but does not complete rest; `start_custom` resets work and starts the supplied duration. No event may consume time twice. Return phase, work elapsed, work remaining, break remaining, completed breaks, and effect names in order. |
| `pause_gate` | Sorted distinct active reasons block breaks; configured exclusions remove matching reasons. No remaining reasons means allow. Clearing one reason cannot clear another. |
| `office_hours` | Weekday index Monday=0. Minutes since local midnight. Start inclusive, end exclusive; an overnight interval's after-midnight portion belongs to the previous weekday. Reject equal start/end instead of guessing all-day. Timezone conversion belongs to the shell. |
| `planned_overlap` | Weekly half-open intervals; expand overnight intervals into the next day. Report pair IDs in sorted order, including Sunday/Monday overlaps. Adjacent endpoints are not overlap. |
| `planned_rest` | Clip away intervals to planned window, merge overlaps, and subtract their union from duration; return credited/remaining milliseconds. |
| `skip_allowed` | Casual allows immediately; balanced requires elapsed rest at least configured delay; hardcore disallows. Planned `skip_today` is allowed in all modes. |
| `snooze_allowed` | `null` is unlimited; reject when either session/day count has reached its limit. Zero disallows. Return `allowed`. |
| `stats` | Positive completed work-session lengths give total, longest, and median milliseconds; empty input returns zeros. App/domain detail cannot add time to total. Screen Score formula is a separate pending decision. |

Run from this file's directory:

```sh
python3 run_contract_tests.py --check-fixtures
python3 run_contract_tests.py --self-test
python3 run_contract_tests.py --progress
cargo build --locked -p awaytime-contract
python3 run_contract_tests.py --operation validate_settings --operation pause_gate --command target/debug/awaytime-contract
# Run the full corpus after remaining operations are implemented:
# python3 run_contract_tests.py --command target/debug/awaytime-contract
```

If the adapter needs arguments, place them after `--command`; for example `--command node /absolute/path/to/adapter.mjs`. Each port implements the adapter, not a second copy of the business rules.

The 50 behavior checkboxes are located in Phases 1, 3, and 6. Native/reliability scenarios AT-01–AT-42 appear alongside their owning phases. A written scenario is not passing evidence.

## Milestones and release tracking

These summary milestones refer to the phase checklists above; they do not replace their completion gates.

- [ ] CORE-01 — Implement the core adapter; pass all 50 behavior cases by the end of Phase 6.
- [ ] UI-01 — Deliver the usable cross-platform desktop app in Phase 2; complete native checks on each target as integrations land through Phase 7.
- [ ] RELEASE-01 — Complete Phase 8 packaging and release checks; publication is a separate release action.
- [ ] M-01 — Phases 0–1: integration proof and stack/support decisions, portable architecture, persistence boundary, test adapter, and break/discipline core.
- [ ] M-02 — Phase 2: usable desktop menu bar/tray, shared settings, break windows, reminders, keyboard controls, and baseline accessibility on each target.
- [ ] M-03 — Phases 3–4: idle recovery, schedules, planned breaks, activity detection, calendar, and Focus.
- [ ] M-04 — Phases 5–6: wellness, customization, localization, statistics, and data controls.
- [ ] M-05 — Phase 7: automations, external scripting, advanced lock-screen/screensaver integrations.
- [ ] M-06 — Phase 8: full shared-corpus comparison, platform native/UI verification, performance/soak checks, installers, and distribution.

## Definition of done for each feature

- [ ] DONE-01 — Expected behavior and boundary cases specified; ambiguous policies documented.
- [ ] DONE-02 — Implementation has no hidden dependence on real time in core tests.
- [ ] DONE-03 — Relevant portable tests pass against real core, with command/revision/evidence logged.
- [ ] DONE-04 — Relevant platform/UI tests pass on the support matrix, including permission failure.
- [ ] DONE-05 — Persisted settings migrate and unsupported capabilities are handled.
- [ ] DONE-06 — User-facing labels and accessibility behavior verified.
- [ ] DONE-07 — Feature ID checked and evidence recorded only after all applicable gates pass.

## Decisions still to resolve

Resolve each decision before the first phase that depends on it. DEC-01/DEC-02: Phase 0; timer/discipline policies in DEC-03/DEC-04: Phase 1; schedule policies: Phase 3; score/retention policies: Phase 6; performance/distribution: Phase 8; optional commercial work in DEC-06: after the first release.

- [ ] DEC-01 — App name, UI identity, and accepted cross-platform stack. Tauri 2 + Rust + React/TypeScript + Vite proposed pending Phase 0 integration proof; foundation set up; native proof in progress; application name AwayTime selected; visual identity and stack acceptance pending.
- [ ] DEC-02 — Minimum OS support, architectures, Linux environments, and exact platform capability matrix. Define the macOS, Windows, and Linux support scope using integration-proof evidence.
- [ ] DEC-03 — Score formula, early-end threshold, idle rest thresholds, planned-break collision window, and minimum useful remainder.
- [ ] DEC-04 — Quota reset policy on timezone travel, pause semantics during active rest, and long-break counting after skipped/custom/planned breaks.
- [ ] DEC-05 — Retention period, performance budgets, and update/distribution strategy.
- [ ] DEC-06 — Optional commercial scope after the core app works.

## Test run record

Record validation results here. Preparation checks verify fixtures and the harness, not app behavior. Application features require passing results from a real adapter and applicable native/UI checks.

| Date | Check | Result | Application behavior verified? |
| --- | --- | --- | --- |
| 2026-10-13 | Fixture structures/links and harness | 50 cases validated; 15 harness self-tests pass | No |
| 2026-10-13 | Rust foundation | 15 tests pass across core/runtime/persistence, including interrupted child process and concurrent writers | Foundation only |
| 2026-10-13 | Core contract subset | 11/50 shared cases pass; 39 pending | Settings/pause queries |
| 2026-10-13 | Adapter failure protocol | Malformed JSON, boolean version, unsupported operation, wrong reason types, oversized input, and missing adapter fail correctly | Protocol only |
| 2026-10-13 | TypeScript/Vite, native compile, and Clippy | Pass on arm64/macOS 26.5.1; development app bundle built | Compile evidence only |
| 2026-10-13 | Mac native interaction | Start/pause/resume/stop, idle observations, two preview windows, main-window hide, persisted work, quit, and session restoration observed | Partial integration evidence; support matrix lists gaps |
| 2026-10-13 | Hosted CI | Mac/Windows/Ubuntu workflow configured; not executed | No |
