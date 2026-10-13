# Data contract, version 1; snapshot schema, version 2

This language-neutral specification defines the application adapter, time values, snapshot format, and storage requirements.

## JSON test process

Each invocation reads one JSON object on stdin, writes one JSON object on stdout, and exits. Version must be integer `1`. Input is an object. Diagnostics go to stderr. Invalid input or an unimplemented operation returns process status `2` with an error object on stderr and no successful stdout result.

```json
{"version":1,"operation":"validate_settings","input":{"work_ms":1200000,"short_break_ms":20000,"snooze_limit":3}}
```

```json
{"errors":[],"valid":true}
```

The `pause_gate` operation accepts string arrays `reasons`/`excluded` and returns `allowed` plus sorted distinct remaining `reasons`. Limits are one MiB per accepted request. The runner treats malformed JSON, wrong types, missing expected fields, nonzero exit, and timeout as failures. Application values must be computed from core rules, independent of the fixture file.

## Integer/time rules

Durations are integer milliseconds; valid values fit `0...9007199254740991`, with work/break durations strictly positive. This bound preserves exact values in ports using JavaScript numbers. Booleans, fractional durations, strings, and overflow are invalid. An absent/null snooze limit means unlimited. Long-break duration and frequency are either both absent/null or both positive. Errors are sorted field names.

Clock samples have `monotonic_ms`, `wall_clock_ms`, `local_day`, `weekday`, `minute`, and `timezone`. Wall milliseconds are signed within the same exact-integer bound. Local day uses a valid Gregorian `YYYY-MM-DD`; weekday is Monday=0; minutes use `0...1439`. Do not persist a monotonic reading across process restart; it has no meaning on a different boot/platform. Floor fractional elapsed/epoch milliseconds in the native clock adapter; fractional user-supplied durations remain errors.

## Snapshot envelope

```json
{
  "schema_version": 2,
  "settings": {
    "work_ms": 1200000,
    "short_break_ms": 20000,
    "long_break_ms": 300000,
    "long_every": 3,
    "snooze_limit": 4
  },
  "session": {
    "phase": "working",
    "work_elapsed_ms": 1234,
    "cycle": 2,
    "quota_day": "2026-10-09",
    "snoozes_today": 2,
    "snoozes_session": 1,
    "pause_reasons": []
  }
}
```

All known values validate before load/save. Unknown optional fields at root, settings, and session levels are retained in their original locations. Known fields override extension collisions when encoding. Missing optional quota day, pause reasons, and disabled long-break fields are allowed. A paused session without reasons restores conservatively as a manual pause. Settings, quota, cycle, elapsed values, and metadata round-trip; migration never reads current time.

Compatibility schema `1` uses integer `work_seconds` and `short_break_seconds`. Migration multiplies these by 1000 with range checks and retains unrelated fields. Ambiguous old/new duration keys are rejected. Schema versions outside `1`/`2` fail clearly. Future schemas never fall back to an older backup or get overwritten.

`snapshot.json` is the current file; `snapshot.previous.json` is the previous valid file. A missing store returns no snapshot without writing. Corrupt-primary recovery reports `backup`; no UI or automatic reset hides that recovery. A backup contains the previous committed version, so recovery can lose the most recent update; the app must report recovery to its user.

Full active-break state, scheduling state, and usage history are intentionally not represented by the Phase 0 session schema. Add versioned fields/migrations and tests when the actual engine requires them. A phase label is not sufficient to resume an active break.
