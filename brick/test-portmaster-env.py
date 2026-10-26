#!/usr/bin/env python3
"""Exercise the stock-firmware compatibility hooks with real child processes."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

HOOK = Path(__file__).parent / 'card/Slot/System/portmaster-env.sh'


class EnvironmentTests(unittest.TestCase):
    def test_missing_pkill_gets_fallback(self):
        result = subprocess.run(['bash', '-c', '''
PATH=
source "$1"
declare -F pkill || exit 10
pkill -f unrelated
[ "$?" = 2 ]
''', 'test', str(HOOK.resolve())], capture_output=True, text=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_patch_output_keeps_gui_stream_stderr_and_failure_status(self):
        with tempfile.TemporaryDirectory(prefix='slot card ') as tmp:
            card = Path(tmp)
            script = card / 'Data/ports/balatro/tools/patchscript'
            script.parent.mkdir(parents=True)
            script.write_text('echo "patch progress"; echo "patch mismatch" >&2; exit 7\n')
            env = dict(os.environ, SLOT_ROOT=str(card / 'Slot'), BASH_ENV=str(HOOK.resolve()))
            result = subprocess.run(['bash', str(script)], env=env, capture_output=True, text=True, timeout=5)
            self.assertEqual(result.returncode, 7)
            self.assertEqual(result.stdout, 'patch progress\npatch mismatch\n')
            self.assertEqual((card / 'Slot/Logs/balatro-patch.log').read_text(), result.stdout)
            result = subprocess.run(['bash', '-c', 'echo unrelated'], env=env, capture_output=True, text=True, timeout=5)
            self.assertEqual(result.stdout, 'unrelated\n')
            self.assertEqual((card / 'Slot/Logs/balatro-patch.log').read_text(), 'patch progress\npatch mismatch\n')

    def test_cleanup_kills_only_named_helpers_in_own_process_group(self):
        with tempfile.TemporaryDirectory() as tmp:
            helper = Path(tmp) / 'gptokeyb'
            shutil.copyfile('/bin/sleep', helper)
            helper.chmod(0o755)
            outside = subprocess.Popen([str(helper), '30'], start_new_session=True)
            try:
                result = subprocess.run(['bash', '-c', '''
source "$1"
"$2" 30 & helper=$!
sleep 0.05
slot_portmaster_pkill -9 -f gptokeyb || exit 10
wait "$helper"
[ "$?" = 137 ] || exit 11
slot_portmaster_pkill -f unrelated
[ "$?" = 2 ] || exit 12
''', 'test', str(HOOK.resolve()), str(helper)], start_new_session=True,
                    capture_output=True, text=True, timeout=5)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIsNone(outside.poll(), 'unrelated process group was signalled')
            finally:
                outside.terminate()
                outside.wait(timeout=5)


if __name__ == '__main__':
    unittest.main()
