# Support and verification matrix

Updated: 2026-10-13 (Asia/Kathmandu)

Cross-platform desktop support for macOS, Windows, and Linux is required. **Tauri 2 + Rust + React/TypeScript + Vite is proposed pending integration proof.** Planned targets are not verified support. DEC-02 will settle minimum versions, architectures, Linux distributions/desktop environments, and the exact capability matrix.

## Planned desktop targets

| Target | Planned scope | Evidence/status |
| --- | --- | --- |
| macOS | Development deployment metadata targets 13+; minimum runtime floor and Intel coverage remain unverified | Foundation and arm64 shell verified locally on macOS 26.5.1; complete proof pending |
| Windows | Candidate desktop target: Windows 10/11 x64; final minimum and architectures remain open | Source and CI configured; native build/runtime evidence pending |
| Linux | Candidate build target: Ubuntu 24.04; X11 and Wayland need separate runtime checks | X11 idle source and CI configured; runtime evidence pending; sleep/wake and focus restoration unavailable |

Before stack acceptance, run the [Phase 0 integration proof](ARCHITECTURE.md#integration-proof-before-stack-acceptance) on all three desktop platforms. Attach revision, OS/environment, hardware, permissions, results, limitations, and resource measurements. Resolve required failures or revise the proposal before marking DEC-01/DEC-02 complete.

## Proposed capability checks

Every entry below is a requirement to investigate, not a claim of current platform support. Record each platform/environment as supported, unavailable, permission-required, or permission-denied only after checking it.

| Capability | Acceptance evidence needed |
| --- | --- |
| Tray/menu controls and background lifecycle | Commands work without an open settings window; quit cleans up; document tray differences |
| Multi-display break windows and focus restoration | Two-monitor coverage, fullscreen/virtual-desktop behavior, disconnect, and wake verified |
| Idle observation and sleep/wake | Correct events, denied/unsupported handling, no stale pause state or duplicate breaks |
| Shared engine and local storage | Matching implemented contract cases; later migration, atomic-write/recovery, and full behavior checks on each target |
| Shortcuts, notifications, login startup, and accessibility | Verify separately on each target as Phase 2 features land; use each platform's screen reader |
| Activity, calendar, and Focus integration | Verify per feature in Phase 4; equivalents are not assumed across platforms |
| AppleScript and Apple Shortcuts | macOS-specific planned integrations; no Windows/Linux parity claim |
| Distribution and updates | Platform-appropriate installers, signing where applicable, macOS notarization, and installed-app verification in Phase 8 |

## Local evidence — 2026-10-13

Host: macOS 26.5.1, Apple Silicon arm64, Xcode 26.5, Rust 1.99.0, Node 22.22.1. The foundation is committed to the project repository. Raw automated logs and host metadata are under `artifacts/phase0/`; these are local generated evidence, not hosted CI results.

| Check | Evidence | Status |
| --- | --- | --- |
| Core/runtime/persistence | 15 Rust tests passed; migration, recovery, concurrency, and interrupted-process write covered | Verified locally |
| Shared contract subset | 11 settings/pause-gate cases passed; 39 operations/cases remain pending | Verified locally |
| Fixture runner | All 50 structures/links validated; 15 harness self-tests passed; missing adapter and malformed request guards exercised | Preparation/protocol verified |
| Frontend and shell | TypeScript/Vite build, native compile, Clippy, and development app bundle passed | Verified locally on arm64 |
| Native session UI | Start, pause, resume, stop, saved-state restoration, and live idle observations seen in the running app | Basic runtime evidence |
| Display preview | Two displays enumerated; two native preview windows reported active; preview rendered on the observed display | Full two-display coverage, Spaces/fullscreen, rotation, disconnect, and wake checks pending |
| Window lifecycle | Main window hidden with app still running; quit wrote accrued work and the app exited; relaunch restored elapsed work without downtime | Basic runtime evidence; tray and Dock-reopen verification pending |
| Sleep/wake | macOS workspace observer implemented; no real suspend/resume performed | Pending |
| Windows | Native idle, power-message observer, and focus calls implemented | Uncompiled/unexecuted locally; CI pending |
| Linux | X11 idle counter implemented; Wayland idle, sleep/wake, and focus restoration explicitly unavailable | Linux build/runtime proof pending |
| Hosted CI | macOS/Windows/Ubuntu check matrix and per-revision artifact upload configured | Not executed |
| Resources | Development bundle about 27 MiB. Short main-process sample: median CPU 0.1%, max RSS 116.25 MiB; excludes WebKit helpers and is not an idle/battery budget | Total process memory, sustained idle CPU, detector churn, latency, and battery measurements pending |

## Verification status

Tauri remains proposed. The full shared corpus and applicable native/UI checks are required on every agreed desktop target under AR-10/AT-36 before release. Current Mac compile and partial runtime evidence do not certify Windows/Linux support.

DEC-01, DEC-02, PL-09, AT-37, and PH-00 remain open. Required Linux services, full macOS lifecycle/display/sleep checks, native Windows/Linux execution, and resource evidence must be resolved before stack acceptance. Record results per OS version, architecture, and desktop environment.

## Development distribution

Successful main builds publish experimental pre-releases with macOS arm64/x64 DMGs, a Windows x64 EXE, and a Linux x64 AppImage built on Ubuntu 22.04. All Phase 0 check jobs and installer builds must pass before publication. macOS uses ad-hoc signing without notarization; Windows is unsigned. Hosted packaging results establish build evidence only. AT-42 and PH-00 remain open until native runtime proof is recorded. See [the pre-release notes](PRE_RELEASE_NOTES.md).
