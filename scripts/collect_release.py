#!/usr/bin/env python3
"""Collect a single installer, or require all four before publication."""
from pathlib import Path
import argparse
import os
import shutil

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'artifacts/release'
TARGETS = {
    'aarch64-apple-darwin': 'dmg',
    'x86_64-apple-darwin': 'dmg',
    'x86_64-pc-windows-msvc': 'exe',
    'x86_64-unknown-linux-gnu': 'AppImage',
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verify', action='store_true')
    args = parser.parse_args()
    if args.verify:
        expected = {f'AwayTime-{target}.{ext}' for target, ext in TARGETS.items()}
        actual = {p.name for p in OUTPUT.iterdir() if p.is_file()}
        if actual != expected:
            raise SystemExit(f'Installer set mismatch: missing={expected - actual}, unexpected={actual - expected}')
        if any((OUTPUT / name).stat().st_size == 0 for name in expected):
            raise SystemExit('Empty installer')
        print('All four installers present')
        return
    target = os.environ['RELEASE_TARGET']
    extension = os.environ['RELEASE_EXTENSION']
    if TARGETS.get(target) != extension:
        raise SystemExit('Unknown target or installer type')
    candidates = list((ROOT / 'target' / target / 'release/bundle').rglob(f'*.{extension}'))
    if len(candidates) != 1 or candidates[0].stat().st_size == 0:
        raise SystemExit(f'Expected one nonempty {extension} installer; found {candidates}')
    OUTPUT.mkdir(parents=True, exist_ok=True)
    destination = OUTPUT / f'AwayTime-{target}.{extension}'
    shutil.copy2(candidates[0], destination)
    print(destination)


if __name__ == '__main__':
    main()
