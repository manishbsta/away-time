# Architecture and working decisions

Updated: 2026-10-13 (Asia/Kathmandu)

The app targets macOS, Windows, and Linux. Tauri is proposed pending integration proof. The Rust foundation and Tauri proof shell are implemented. Cross-platform runtime proof and final stack acceptance remain pending.

## Module dependencies

`desktop → persistence/runtime → core`; persistence uses the runtime storage interface. The contract executable depends only on core. React submits typed commands to the desktop gateway. The core boundary is enforced by `scripts/check_phase0.py`; elapsed clocks and platform APIs stay outside the core.

## Commands, time, and effects

The core must receive typed commands and clock samples and return state plus effects as data. It must not read a clock, open a file, show a window, or execute a script. For example, starting a break returns a request to show break windows; the platform module executes it.

One serialized command gateway must handle UI, tray, keyboard, and script actions. Reject stale command sequences and backward elapsed-time samples; wall-clock changes must not alter accrued work. Cleanup must own and release subscriptions, windows, audio, and pending actions on stop/quit, including resources acquired during shutdown.

Inject elapsed integer milliseconds from a monotonic clock separately from local calendar samples. Keep schedule days, weekdays, minutes, and timezone identifiers in the language-neutral data contract. Do not persist process-local time/sequence baselines across restarts. Saved elapsed work stays frozen until a new engine command advances it; sleep/rest and quota policies remain Phase 3 work.

## Platform adapters

Define separate services for clock, activity, idle, displays, notifications, audio, persistence, Focus, calendar, automation, and scripting. Represent each capability as supported, unavailable, permission-required, or permission-denied. Platform modules own OS calls and cleanup; request permissions only for enabled dependent features. Service interfaces and capability statuses exist. The desktop shell configures storage, displays, idle observations, and platform sleep/wake where available. Activity, notifications, audio, Focus, calendar, automation, scripting, and global shortcuts report unavailable in this proof.

## Local-first data

Implement the versioned JSON contract, migration rules, unknown optional fields, atomic writes, and previous-snapshot recovery. Full active-break state requires a documented schema extension; a saved phase label alone cannot resume a break. SQLite history remains Phase 6 work. The file store serializes access with a filesystem lock, synchronizes temporary files before replacement, and synchronizes directories on Unix. Snapshot migration and validation run in the pure codec. Future schemas never fall back to an older backup or get overwritten. Hardware power-loss durability and Windows directory-flush behavior require native evidence.

## Cross-platform stack proposal

**Tauri 2 + Rust + React/TypeScript + Vite is proposed pending integration proof.** Resolve DEC-01 after the integration proof below; resolve DEC-02 with exact platform versions, architectures, environments, and capability evidence.

| Responsibility | Proposed choice | Example |
| --- | --- | --- |
| Desktop shell | Tauri 2 | Own windows, tray/menu controls, and app lifecycle |
| Shared interface | React + TypeScript, built with Vite | Render settings, countdowns, and statistics from engine state |
| Shared behavior and background runtime | Rust | Own break timing, schedules, snooze quotas, and session recovery independently of open screens |
| Operating-system services | Separate platform modules, with native code where needed | Translate idle and sleep/wake observations into common engine events |
| Local data | Versioned JSON settings/snapshots; SQLite history in Phase 6 | Restore a saved work session and query completed-break history |

Tauri uses operating-system webviews and a separate application process. Its smaller bundled runtime makes it a candidate for an always-running utility; actual CPU, memory, battery use, and interface differences need measurement. Rust introduces a new language/toolchain, and deep operating-system integrations still require platform work. See the official [architecture](https://v2.tauri.app/concept/architecture/) and [process model](https://v2.tauri.app/concept/process-model/) documentation.

The proposed dependency direction is `shared screens → command gateway → Rust runtime/core`, with platform modules supplying observations and executing effects. The core stays independent of Tauri, React, filesystem access, and OS calls. A 20-minute work session must continue when the settings window closes. UI, tray, shortcuts, and scripts must submit to the same serialized command path.

### Integration proof before stack acceptance

Complete this proof in Phase 0 before accepting the stack or starting dependent phases. Run a small working app on macOS, Windows, and selected Linux environments; record the exact OS, hardware, display configuration, permissions, revision, observed behavior, and limitations in the support matrix.

1. **Tray and lifecycle:** start/pause/resume/quit work through tray/menu controls; closing settings keeps the engine running; quitting releases owned resources. Record environment-specific tray behavior.
2. **Break windows:** cover two monitors, restore previous focus, and handle display disconnect and sleep/wake. Check macOS Spaces/fullscreen, Windows fullscreen, and selected Linux window-management environments separately.
3. **Idle detection:** observe active/idle transitions, handle unsupported or denied access, and verify idle observations reach the shared command path without stale pause reasons.
4. **Sleep/wake recovery:** keep elapsed-time decisions independent of wall-clock changes, deliver wake once, and prevent duplicate break windows. Full rest-credit policy remains Phase 3 work.
5. **Behavior and resources:** implement a Rust JSON test adapter and pass the settings/pause-gate fixtures (11 cases). Measure background CPU/memory, window latency, and battery impact where feasible; record results without claiming a performance advantage from framework choice alone.

Accept the stack only when the proof works within the agreed platform capability matrix and remaining limits are documented. If a required capability fails, resolve that failure or revise the stack/support proposal before acceptance. A restricted Linux environment or unsupported integration must not be presented as full support. The proof is **in progress on macOS**; see the support matrix for current evidence and missing checks. It is smaller than the full behavior, reliability, and release gates in later phases.

### Implementation after a successful proof

- Implement the states/commands/effects boundary, injected elapsed and calendar time, cleanup ownership, and explicit supported/unavailable/permission statuses.
- Implement shared behavior in Rust using the specified fixture IDs and expected results. Remaining cases stay pending until implemented; complete the full corpus on every agreed target and AR-10/AT-36 before release.
- Follow the versioned data contract, unknown optional fields, migration rules, and recovery behavior. Verify import/export and atomic-write/recovery behavior on every target; platform-specific filesystem guarantees need their own implementation and checks.
- Keep integrations behind platform modules. For example, AppleScript and Apple Shortcuts remain macOS-specific; other platforms expose only verified equivalents. Hide unsupported controls and explain permission failures.
- Keep snapshots in JSON and add SQLite usage history later. No server, account, or commercial licensing is required for core reminders.
- The application name is **AwayTime**, with development identifier `dev.awaytime.desktop`. Visual identity, production identifiers, and minimum OS support remain open.

## Phase 0 limits

Project setup, the 11-case adapter subset, pure snapshot codec, storage recovery tests, React build, and Mac shell compile/build are verified locally. The shell supports manual preview windows and basic command state; automatic break timing and full interval-session behavior belong to Phase 1. Global shortcut/script entry points and full native cleanup integration remain open under AR-08/AR-11. Windows/Linux runtime proof, macOS sleep/wake/display-change checks, resource budgets, and hosted CI remain pending. DEC-01/DEC-02, PL-09, AT-37, and PH-00 are not complete.
