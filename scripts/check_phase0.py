#!/usr/bin/env python3
"""Run Phase 0 checks and save actual output for this host/revision."""
from pathlib import Path
import datetime
import json
import os
import platform
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
ARTIFACTS = ROOT / 'artifacts' / 'phase0'
ARTIFACTS.mkdir(parents=True, exist_ok=True)

def run(name, command, expected=0):
    result = subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (ARTIFACTS / f'{name}.log').write_text(result.stdout)
    print(result.stdout, end='')
    if result.returncode != expected:
        raise SystemExit(f'{name}: expected exit {expected}, got {result.returncode}')

# Fail if the pure core acquires ambient platform/time dependencies.
for path in (ROOT / 'crates/awaytime-core/src').glob('*.rs'):
    text = path.read_text()
    for forbidden in ['std::time::', 'std::fs::', 'std::env::', 'tauri::', 'chrono::', 'windows_sys::', 'objc2::']:
        if forbidden in text:
            raise SystemExit(f'Core boundary violation: {path}: {forbidden}')
run('fixtures', [sys.executable, 'run_contract_tests.py', '--check-fixtures'])
run('harness', [sys.executable, 'run_contract_tests.py', '--self-test'])
run('format', ['cargo', 'fmt', '--all', '--', '--check'])
run('foundation', ['cargo', 'test', '--locked', '--workspace', '--exclude', 'awaytime-desktop'])
run('adapter-build', ['cargo', 'build', '--locked', '-p', 'awaytime-contract'])
executable = ROOT / 'target/debug' / ('awaytime-contract.exe' if os.name == 'nt' else 'awaytime-contract')
run('contract-subset', [sys.executable, 'run_contract_tests.py', '--operation', 'validate_settings', '--operation', 'pause_gate', '--command', str(executable)])
for name, request in [
    ('malformed', 'not JSON'),
    ('bad-version', '{"version":true,"operation":"validate_settings","input":{}}'),
    ('unimplemented', '{"version":1,"operation":"interval_session","input":{}}'),
    ('bad-reasons', '{"version":1,"operation":"pause_gate","input":{"reasons":[true],"excluded":[]}}'),
    ('oversized', ' ' * 1_048_577),
]:
    result = subprocess.run([str(executable)], input=request, text=True, capture_output=True, timeout=5)
    if result.returncode != 2 or result.stdout or not isinstance(json.loads(result.stderr).get('error'), str):
        raise SystemExit(f'Adapter failure protocol failed: {name}')
run('missing-adapter', [sys.executable, 'run_contract_tests.py'], expected=2)
run('web-build', ['npm.cmd' if os.name == 'nt' else 'npm', 'run', 'build'])
run('native-compile', ['cargo', 'check', '--locked', '-p', 'awaytime-desktop'])
revision = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=ROOT, capture_output=True, text=True)
(ARTIFACTS / 'environment.json').write_text(json.dumps({
    'recorded_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'platform': platform.platform(), 'architecture': platform.machine(),
    'revision': revision.stdout.strip() if revision.returncode == 0 else 'uncommitted workspace',
    'runtime_proof': 'Native interaction, multiple displays, real sleep/wake, and resource measurements require separate evidence',
}, indent=2) + '\n')
print('Phase 0 automated checks passed. Native runtime proof remains a separate gate.')
