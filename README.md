# AwayTime

An original screen-break app for macOS, Windows, and Linux. Phase 0 now has a Rust foundation, a React/TypeScript interface, and a Tauri desktop integration proof. Automatic break scheduling belongs to Phase 1. **Tauri 2 remains proposed pending the cross-platform integration proof; PH-00 is open.**

## Stack and modules

- `crates/awaytime-core`: settings validation, pause decisions, injected time, typed commands/effects, and the pure snapshot codec. No clock reads, filesystem access, or platform calls.
- `crates/awaytime-runtime`: serialized command gateway, replaceable service interfaces, explicit capabilities, and resource ownership.
- `crates/awaytime-persistence`: local JSON snapshots, schema migration/recovery, file locking, and atomic replacement.
- `crates/awaytime-contract`: executable JSON adapter for `validate_settings` and `pause_gate`.
- `src-tauri`: desktop shell, tray controls, display previews, and platform services.
- `src`: shared React/TypeScript interface, built with Vite.

Settings and sessions are local; no server or account is needed. SQLite history is Phase 6 work. See [FEATURES.md](FEATURES.md) for the roadmap.

## Develop

Use Node 22.12+ and the Rust toolchain pinned in `rust-toolchain.toml`. Install the [Tauri system prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS. The dependency lockfiles are checked in.

```sh
npm ci
npm run tauri -- dev
```

To build a development app on macOS:

```sh
npm run tauri -- build --debug --bundles app
```

The app bundle is generated under `target/debug/bundle/macos/`. This is a development artifact; release signing, notarization, and installers are Phase 8 work. The development profile omits debug symbols and incremental caches to limit disk use.

## Verify

```sh
npm run check
cargo clippy --locked --workspace --exclude awaytime-desktop -- -D warnings
cargo clippy --locked -p awaytime-desktop -- -D warnings
python3 run_contract_tests.py --progress
```

`npm run check` enforces the core boundary, validates all 50 fixtures, runs harness/foundation tests, builds the adapter, runs its 11 implemented contract cases and error protocol, builds the interface, and compile-checks the desktop shell. Logs and host/revision metadata go to `artifacts/phase0/`. CI is configured for macOS, Windows, and Ubuntu 24.04; hosted execution is pending.

The remaining 39 cases are specifications for later phases. Unsupported adapter operations fail with status 2. To select the implemented subset manually:

```sh
cargo build --locked -p awaytime-contract
python3 run_contract_tests.py --operation validate_settings --operation pause_gate --command target/debug/awaytime-contract
```

On Windows, the executable has the `.exe` suffix. Tests of the harness are preparation checks, not app behavior.

## Try the integration proof

Start/pause/resume/stop a session from the app or tray. Closing the main window hides it while the background session continues. Open it again from the tray; quit explicitly from the tray or application menu. A restored session keeps its saved elapsed time without counting app downtime.

Preview a break to create borderless windows on connected displays. The preview closes after 15 seconds, and its return button or Escape can dismiss it early. It does not count as a completed break. Native idle counters are observations only; automatic idle rest credit and scheduling remain later work.

macOS idle and workspace sleep/wake observers are implemented. Windows adapters are implemented but require native execution. Linux X11 idle queries are implemented; Wayland idle, Linux sleep/wake, and Linux focus restoration report unavailable until the appropriate integrations are added. [The support matrix](docs/SUPPORT_MATRIX.md) separates implemented code from verified behavior.

## Storage

The desktop shell uses the platform app-data directory for `dev.awaytime.desktop`. Loading a missing store creates no data. Commands save versioned `snapshot.json`; `snapshot.previous.json` keeps the previous valid commit. A recovered backup is reported in the app. Corrupt files without a valid backup and incompatible future schemas produce visible errors and prevent session changes.

Tests cover migration, unknown optional fields, backup recovery, concurrent writers, and process interruption before commit. Hardware power-loss guarantees and Windows/Linux native filesystem behavior still need evidence. Full active-break state is not represented by the Phase 0 snapshot.

See [the data contract](docs/DATA_CONTRACT.md), [architecture](docs/ARCHITECTURE.md), and [support matrix](docs/SUPPORT_MATRIX.md).

## License

AwayTime is licensed under the [GNU General Public License v3.0](LICENSE).
