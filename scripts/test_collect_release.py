"""Exercise incomplete-release rejection and installer discovery in isolation."""
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import collect_release


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.output = self.root / 'artifacts/release'
        self.output.mkdir(parents=True)
        for name, value in [('ROOT', self.root), ('OUTPUT', self.output)]:
            patcher = patch.object(collect_release, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)

    def verify(self):
        with patch('sys.argv', ['collect_release.py', '--verify']):
            collect_release.main()

    def populate(self):
        for target, extension in collect_release.TARGETS.items():
            (self.output / f'AwayTime-{target}.{extension}').write_bytes(b'installer')

    def test_requires_every_platform(self):
        self.populate()
        self.verify()
        (self.output / 'AwayTime-x86_64-unknown-linux-gnu.AppImage').unlink()
        with self.assertRaises(SystemExit):
            self.verify()

    def test_rejects_empty_and_unexpected_assets(self):
        self.populate()
        installer = self.output / 'AwayTime-x86_64-pc-windows-msvc.exe'
        installer.write_bytes(b'')
        with self.assertRaises(SystemExit):
            self.verify()
        installer.write_bytes(b'installer')
        (self.output / 'unexpected.exe').write_bytes(b'other')
        with self.assertRaises(SystemExit):
            self.verify()

    def test_collects_installer_and_rejects_ambiguous_output(self):
        target = 'x86_64-unknown-linux-gnu'
        bundle = self.root / 'target' / target / 'release/bundle/appimage'
        bundle.mkdir(parents=True)
        (bundle / 'original.AppImage').write_bytes(b'original installer bytes')
        with patch.dict(os.environ, RELEASE_TARGET=target, RELEASE_EXTENSION='AppImage'), patch('sys.argv', ['collect_release.py']):
            collect_release.main()
            self.assertEqual((self.output / f'AwayTime-{target}.AppImage').read_bytes(), b'original installer bytes')
            (bundle / 'duplicate.AppImage').write_bytes(b'other')
            with self.assertRaises(SystemExit):
                collect_release.main()


if __name__ == '__main__':
    unittest.main()
