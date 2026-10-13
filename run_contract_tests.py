#!/usr/bin/env python3
"""Portable app contract runner. No application implementation is included."""
import argparse
import json
import math
from pathlib import Path
import re
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parent
OPERATIONS = {
    'validate_settings', 'interval_session', 'pause_gate', 'office_hours',
    'planned_overlap', 'planned_rest', 'skip_allowed', 'snooze_allowed', 'stats',
}
FEATURE_PREFIXES = {
    'BR', 'RM', 'DS', 'ID', 'SP', 'OH', 'PB', 'WB', 'UX', 'MB', 'ST',
    'AU', 'SC', 'FO', 'AC', 'PL',
}


def reject_constant(value):
    raise ValueError(f'Non-finite JSON number: {value}')


def finite_float(value):
    result = float(value)
    if not math.isfinite(result):
        raise ValueError(f'Overflowing JSON number: {value}')
    return result


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f'Duplicate JSON key: {key}')
        result[key] = value
    return result


def strict_json(value):
    return json.loads(value, parse_constant=reject_constant,
                      parse_float=finite_float, object_pairs_hook=unique_object)


def assert_subset(expected, actual, path='$'):
    """Compare exact scalar types/arrays; recursively allow extra object keys."""
    if type(expected) is not type(actual):
        raise AssertionError(f'{path}: expected {type(expected).__name__}, '
                             f'got {type(actual).__name__}')
    if isinstance(expected, dict):
        for key, value in expected.items():
            if key not in actual:
                raise AssertionError(f'{path}: missing field {key!r}')
            assert_subset(value, actual[key], f'{path}.{key}')
    elif isinstance(expected, list):
        if len(expected) != len(actual):
            raise AssertionError(f'{path}: expected {len(expected)} items, '
                                 f'got {len(actual)}')
        for index, (left, right) in enumerate(zip(expected, actual)):
            assert_subset(left, right, f'{path}[{index}]')
    elif expected != actual:
        raise AssertionError(f'{path}: expected {expected!r}, got {actual!r}')


def load_cases(path, tracker):
    data = strict_json(path.read_text())
    if not isinstance(data, dict) or type(data.get('version')) is not int or data['version'] != 1:
        raise ValueError('Fixture file must have version 1.')
    cases = data.get('cases')
    if not isinstance(cases, list) or not cases:
        raise ValueError('Fixture file must contain nonempty cases.')
    tracker_text = tracker.read_text()
    known_features = {ident for ident in re.findall(r'^- \[[ x]\] ([A-Z]+-\d+) —',
                                                    tracker_text, re.MULTILINE)
                      if ident.split('-')[0] in FEATURE_PREFIXES}
    seen = set()
    for case in cases:
        if not isinstance(case, dict):
            raise ValueError('Case must be an object.')
        ident = case.get('id')
        if not isinstance(ident, str) or not re.fullmatch(r'T-\d{3}', ident):
            raise ValueError(f'Invalid case ID: {ident!r}')
        if ident in seen:
            raise ValueError(f'Duplicate case: {ident}')
        seen.add(ident)
        if not isinstance(case.get('description'), str) or not case['description'].strip():
            raise ValueError(f'{ident}: missing description')
        features = case.get('feature_ids')
        if not isinstance(features, list) or not features:
            raise ValueError(f'{ident}: missing feature links')
        for feature in features:
            if not isinstance(feature, str) or feature not in known_features:
                raise ValueError(f'{ident}: unknown feature ID {feature!r}')
        request = case.get('request')
        if not isinstance(request, dict) or type(request.get('version')) is not int or request['version'] != 1:
            raise ValueError(f'{ident}: invalid request version')
        if request.get('operation') not in OPERATIONS:
            raise ValueError(f'{ident}: unknown operation')
        if not isinstance(request.get('input'), dict):
            raise ValueError(f'{ident}: input must be an object')
        if not isinstance(case.get('expected'), dict) or not case['expected']:
            raise ValueError(f'{ident}: expected must be nonempty object')
        if not re.search(rf'^- \[[ x]\] {re.escape(ident)} —', tracker_text, re.MULTILINE):
            raise ValueError(f'{ident}: missing tracker entry')
    return cases


def call_adapter(command, request, timeout):
    try:
        result = subprocess.run(
            command, input=json.dumps(request, allow_nan=False),
            text=True, capture_output=True, timeout=timeout,
        )
    except subprocess.TimeoutExpired as exc:
        raise AssertionError(f'Adapter timed out after {timeout}s') from exc
    except OSError as exc:
        raise AssertionError(f'Cannot start adapter: {exc}') from exc
    if result.returncode:
        raise AssertionError(f'Adapter exit code {result.returncode}; '
                             f'stderr: {result.stderr[:1000]}')
    try:
        response = strict_json(result.stdout)
    except (ValueError, TypeError) as exc:
        raise AssertionError(f'Adapter must emit one valid JSON object: {exc}') from exc
    if not isinstance(response, dict):
        raise AssertionError('Adapter response must be a JSON object.')
    return response


class HarnessTests(unittest.TestCase):
    def test_extra_fields_allowed(self):
        assert_subset({'state': {'phase': 'work'}},
                      {'state': {'phase': 'work', 'extra': 1}, 'debug': True})

    def test_wrong_value_rejected(self):
        with self.assertRaises(AssertionError):
            assert_subset({'remaining': 1}, {'remaining': 2})

    def test_missing_field_rejected(self):
        with self.assertRaises(AssertionError):
            assert_subset({'remaining': 1}, {})

    def test_bool_cannot_impersonate_integer(self):
        with self.assertRaises(AssertionError):
            assert_subset({'remaining': 1}, {'remaining': True})

    def test_null_cannot_impersonate_zero(self):
        with self.assertRaises(AssertionError):
            assert_subset({'remaining': 0}, {'remaining': None})

    def test_array_order_enforced(self):
        with self.assertRaises(AssertionError):
            assert_subset(['start', 'end'], ['end', 'start'])

    def test_array_length_enforced(self):
        with self.assertRaises(AssertionError):
            assert_subset(['start'], ['start', 'start'])

    def test_nonfinite_json_rejected(self):
        for value in ('NaN', 'Infinity', '-Infinity', '1e999'):
            with self.subTest(value=value), self.assertRaises(ValueError):
                strict_json('{"value": ' + value + '}')

    def test_duplicate_json_keys_rejected(self):
        with self.assertRaises(ValueError):
            strict_json('{"phase": "work", "phase": "break"}')

    def test_fixture_validation(self):
        class TextFile:
            def __init__(self, text):
                self.text = text

            def read_text(self):
                return self.text

        fixture = dict(version=1, cases=[dict(
            id='T-001', description='Test', feature_ids=['BR-01'],
            request=dict(version=1, operation='stats', input={}),
            expected=dict(total_ms=0))])
        tracker = TextFile('- [ ] BR-01 — Example\n- [ ] T-001 — Test\n')
        self.assertEqual(len(load_cases(TextFile(json.dumps(fixture)), tracker)), 1)
        broken_versions = [
            dict(fixture, version=True),
            dict(fixture, cases=fixture['cases'] * 2),
            dict(fixture, cases=[dict(fixture['cases'][0], feature_ids=['BR-99'])]),
            dict(fixture, cases=[dict(fixture['cases'][0], expected={})]),
            dict(fixture, cases=[dict(fixture['cases'][0], request=dict(
                version=1, operation='unknown', input={}))]),
        ]
        for broken in broken_versions:
            with self.subTest(broken=broken), self.assertRaises(ValueError):
                load_cases(TextFile(json.dumps(broken)), tracker)
        with self.assertRaises(ValueError):
            load_cases(TextFile(json.dumps(fixture)), TextFile('- [ ] BR-01 — Example\n'))

    def test_valid_process_protocol(self):
        command = [sys.executable, '-c',
                   'import json,sys; x=json.load(sys.stdin); '
                   'print(json.dumps({"version":x["version"]}))']
        self.assertEqual(call_adapter(command, {'version': 1}, 2), {'version': 1})

    def test_failed_process_rejected(self):
        with self.assertRaisesRegex(AssertionError, 'exit code 3'):
            call_adapter([sys.executable, '-c', 'raise SystemExit(3)'], {}, 2)

    def test_process_timeout_rejected(self):
        with self.assertRaisesRegex(AssertionError, 'timed out'):
            call_adapter([sys.executable, '-c', 'import time; time.sleep(10)'], {}, 0.1)

    def test_invalid_output_rejected(self):
        for code in ('print("hello")', 'print("[]")', 'print("{}{}");'):
            with self.subTest(code=code), self.assertRaises(AssertionError):
                call_adapter([sys.executable, '-c', code], {}, 2)

    def test_missing_executable_rejected(self):
        with self.assertRaisesRegex(AssertionError, 'Cannot start'):
            call_adapter([str(ROOT / 'nonexistent-test-adapter')], {}, 2)


def progress(tracker):
    rows = re.findall(r'^- \[([ x])\] ([A-Z]+-\d+) —', tracker.read_text(), re.MULTILINE)
    features = [(state, ident) for state, ident in rows
                if ident.split('-')[0] in FEATURE_PREFIXES]
    done = sum(state == 'x' for state, _ in features)
    print(f'Desktop feature items verified: {done}/{len(features)}')
    for prefix in sorted(FEATURE_PREFIXES):
        subset = [(state, ident) for state, ident in features if ident.startswith(prefix + '-')]
        print(f'  {prefix}: {sum(state == "x" for state, _ in subset)}/{len(subset)}')
    behavior = [(state, ident) for state, ident in rows if ident.startswith('T-')]
    print(f'Core behavior cases passed and recorded: '
          f'{sum(state == "x" for state, _ in behavior)}/{len(behavior)}')
    print('Counts read checked boxes; they do not independently verify evidence.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--check-fixtures', action='store_true')
    mode.add_argument('--self-test', action='store_true')
    mode.add_argument('--progress', action='store_true')
    parser.add_argument('--operation', action='append', choices=sorted(OPERATIONS), help='Run only the named operation; repeat to select more')
    parser.add_argument('--cases', type=Path, default=ROOT / 'contract_cases.json')
    parser.add_argument('--tracker', type=Path, default=ROOT / 'FEATURES.md')
    parser.add_argument('--timeout', type=float, default=5)
    parser.add_argument('--command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if not math.isfinite(args.timeout) or args.timeout <= 0:
        parser.error('--timeout must be positive and finite')
    if args.command and (args.check_fixtures or args.self_test or args.progress):
        parser.error('Choose a preparation check or --command, not both')
    if args.self_test:
        result = unittest.TextTestRunner(verbosity=2).run(
            unittest.defaultTestLoader.loadTestsFromTestCase(HarnessTests))
        return 0 if result.wasSuccessful() else 1
    try:
        if args.progress:
            progress(args.tracker)
            return 0
        cases = load_cases(args.cases, args.tracker)
    except (OSError, ValueError, TypeError) as exc:
        print(f'Invalid fixtures/tracker: {exc}', file=sys.stderr)
        return 2
    if args.check_fixtures:
        print(f'Validated {len(cases)} case structures and tracker links. '
              'Application behavior was not tested.')
        return 0
    if not args.command:
        print('BLOCKED: no core adapter supplied. Use --command EXECUTABLE [ARGS...]. '
              'No application tests ran.', file=sys.stderr)
        return 2

    if args.operation:
        cases = [case for case in cases if case['request']['operation'] in args.operation]
        if not cases:
            parser.error('Selected operations have no fixtures')
        print(f'Selected {len(cases)} behavioral cases for {args.operation}. Other operations remain pending.')

    class CoreContractTests(unittest.TestCase):
        pass

    for case in cases:
        def run_case(self, case=case):
            actual = call_adapter(args.command, case['request'], args.timeout)
            assert_subset(case['expected'], actual)
        run_case.__doc__ = case['description']
        setattr(CoreContractTests, 'test_' + case['id'].replace('-', '_'), run_case)
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(CoreContractTests))
    print(f'Application contracts: {result.testsRun} run, '
          f'{len(result.failures)} failures, {len(result.errors)} errors. '
          'Tracker checkboxes must be updated with actual evidence.')
    return 0 if result.wasSuccessful() else 1


if __name__ == '__main__':
    raise SystemExit(main())
