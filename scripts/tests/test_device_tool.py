"""Reset sequencing regressions; all serial and flash operations are mocked."""
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / 'device-tool.py'
SPEC = importlib.util.spec_from_file_location('p4desk_device_tool', SCRIPT)
device_tool = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(device_tool)


class FakeSerial:
    events = None

    def __init__(self, **kwargs):
        self.events.append(('create', kwargs))
        self.is_open = False
        self._dtr = True
        self._rts = True

    @property
    def dtr(self):
        return self._dtr

    @dtr.setter
    def dtr(self, value):
        self._dtr = value
        self.events.append(('dtr', value))

    @property
    def rts(self):
        return self._rts

    @rts.setter
    def rts(self, value):
        self._rts = value
        self.events.append(('rts', value))

    def open(self):
        self.events.append(('open', self.dtr, self.rts))
        self.is_open = True

    def close(self):
        self.events.append(('close', self.dtr, self.rts))
        self.is_open = False

    def readline(self):
        return b''


class DeviceToolTests(unittest.TestCase):
    def setUp(self):
        self.events = []
        FakeSerial.events = self.events
        self.fake_serial = SimpleNamespace(Serial=FakeSerial)

    def sleep(self, duration):
        self.events.append(('sleep', duration))

    def test_reset_presets_boot_and_holds_release_before_close(self):
        with patch.dict(sys.modules, serial=self.fake_serial), \
                patch.object(device_tool.time, 'sleep', side_effect=self.sleep), \
                patch('builtins.print'):
            device_tool.reset(SimpleNamespace(port='/dev/cu.TEST_P4'))
        self.assertEqual(self.events[0][1]['port'], None)
        self.assertFalse(self.events[0][1]['rtscts'])
        self.assertFalse(self.events[0][1]['dsrdtr'])
        self.assertIn(('open', False, False), self.events)
        self.assertNotIn(('dtr', True), self.events)
        asserted = self.events.index(('rts', True))
        self.assertEqual(self.events[asserted + 1], ('sleep', 0.1))
        self.assertEqual(self.events[asserted + 2], ('rts', False))
        self.assertLess(self.events.index(('sleep', 0.2)), asserted)
        self.assertLess(self.events.index(('sleep', 0.5)),
                        self.events.index(('close', False, False)))
        self.assertEqual(self.events[-1], ('close', False, False))

    def test_interrupted_reset_releases_en_without_asserting_boot(self):
        def interrupted_sleep(duration):
            self.sleep(duration)
            if duration == 0.1:
                raise KeyboardInterrupt()

        with patch.dict(sys.modules, serial=self.fake_serial), \
                patch.object(device_tool.time, 'sleep', side_effect=interrupted_sleep):
            with self.assertRaises(KeyboardInterrupt):
                device_tool.reset(SimpleNamespace(port='/dev/cu.TEST_P4'))
        self.assertNotIn(('dtr', True), self.events)
        self.assertEqual(self.events[-1], ('close', False, False))

    def test_passive_monitor_opens_without_reset_pulse(self):
        args = SimpleNamespace(worker=True, port='/dev/cu.TEST_P4',
                               reset=False, output=None, duration=1)
        with patch.dict(sys.modules, serial=self.fake_serial), \
                patch.object(device_tool.time, 'monotonic', side_effect=[0, 0, 2]):
            device_tool.monitor(args)
        self.assertIn(('open', False, False), self.events)
        self.assertNotIn(('dtr', True), self.events)
        self.assertNotIn(('rts', True), self.events)
        self.assertEqual(self.events[-1], ('close', False, False))

    def flash_fixture(self, directory):
        root = Path(directory)
        backup = root / 'backup.bin'
        with backup.open('wb') as handle:
            handle.truncate(32 * 1024 * 1024)
        backup.with_suffix('.json').write_text(json.dumps({
            'status': 'device_md5_and_file_sha256_verified', 'sha256': 'fixture',
        }))
        (root / 'app.bin').write_bytes(b'fixture')
        (root / 'flasher_args.json').write_text(json.dumps({
            'flash_files': {'0x20000': 'app.bin'}, 'write_flash_args': [],
        }))
        return SimpleNamespace(backup=str(backup), build_dir=str(root),
                               python='/fixture/idf-python',
                               port='/dev/cu.TEST_P4', baud='460800')

    def test_failed_flash_never_requests_startup_reset(self):
        with tempfile.TemporaryDirectory() as directory:
            args = self.flash_fixture(directory)
            with patch.object(device_tool, 'sha256_file', return_value='fixture'), \
                    patch.object(device_tool, 'run_tool', side_effect=SystemExit(1)), \
                    patch.object(device_tool.subprocess, 'call') as reset_worker, \
                    patch('builtins.print'):
                with self.assertRaises(SystemExit):
                    device_tool.flash(args)
                reset_worker.assert_not_called()

    def test_successful_flash_uses_idf_python_after_esptool_releases_port(self):
        order = []

        def flash_done(arguments, python, port):
            self.assertEqual(arguments[:2], ['--after', 'no-reset'])
            order.append('flash-finished')

        def reset_worker(command):
            self.assertEqual(command, ['/fixture/idf-python', str(SCRIPT),
                                       'reset', '--port', '/dev/cu.TEST_P4'])
            order.append('reset-worker')
            return 0

        with tempfile.TemporaryDirectory() as directory:
            args = self.flash_fixture(directory)
            with patch.object(device_tool, 'sha256_file', return_value='fixture'), \
                    patch.object(device_tool, 'run_tool', side_effect=flash_done), \
                    patch.object(device_tool.subprocess, 'call', side_effect=reset_worker), \
                    patch('builtins.print'):
                device_tool.flash(args)
        self.assertEqual(order, ['flash-finished', 'reset-worker'])


if __name__ == '__main__':
    unittest.main()
