#!/bin/zsh
set -euo pipefail
TASK_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TASK_EXECUTABLE="$TASK_ROOT/dist/P4Desk.app/Contents/MacOS/P4Desk"
if [[ ! -x "$TASK_EXECUTABLE" ]]; then
  "$TASK_ROOT/scripts/build-macos.sh"
fi
TASK_EVIDENCE="${1:-$TASK_ROOT/desktop/macos/acceptance/mac27-arm64-probes.json}"
python3 - "$TASK_EXECUTABLE" "$TASK_EVIDENCE" "$TASK_ROOT/tests/fixtures/snapshot-v1.json" <<'PY'
from pathlib import Path
import datetime, hashlib, json, platform, re, subprocess, sys, tempfile

executable, destination, fixture = map(Path, sys.argv[1:])
results = {}
passed = True
package = fixture.parents[2] / 'desktop' / 'macos'
tests = subprocess.run(['swift', 'test', '--package-path', str(package)], capture_output=True, text=True, timeout=120)
counts = [int(count) for count in re.findall(r'Executed (\d+) tests', tests.stdout + tests.stderr)]
test_count = max(counts, default=0)
results['swift_tests'] = {'exit_code': tests.returncode, 'test_count': test_count, 'passed': tests.returncode == 0 and test_count >= 19}
passed = results['swift_tests']['passed']
print(json.dumps({'probe': 'swift_tests', **results['swift_tests']}))
for key, argument in [('protocol', '--self-test'), ('synthetic_jpeg', '--probe-jpeg'), ('virtual_display', '--probe-virtual-display')]:
    result = subprocess.run([str(executable), argument], capture_output=True, text=True, timeout=20)
    rows = []
    for line in result.stdout.splitlines():
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        rows.append(value)
        print(json.dumps(value, ensure_ascii=False))
    results[key] = {'exit_code': result.returncode, 'observations': rows}
    passed = passed and result.returncode == 0 and bool(rows)

resources = executable.parent.parent / 'Resources'
with tempfile.TemporaryDirectory(prefix='p4desk-bundled-font-') as directory:
    output = Path(directory) / 'fixture.p4f'
    helper, font = resources / 'p4desk-fontpack', resources / 'HarmonyOS_Sans_SC_Regular.ttf'
    baked = subprocess.run([str(helper), 'bake', '--font', str(font), '--snapshot', str(fixture),
        '--output', str(output), '--sizes', '18,22,28,36'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=30)
    if baked.returncode == 0:
        checked = subprocess.run([str(helper), 'validate', '--input', str(output), '--snapshot', str(fixture)],
            capture_output=True, text=True, timeout=10)
        value = json.loads(checked.stdout) if checked.returncode == 0 else {'valid': False}
        results['bundled_fontpack'] = value
        passed = passed and checked.returncode == 0 and value.get('valid') is True
        print(json.dumps({'probe': 'bundled_fontpack', **value}))
    else:
        results['bundled_fontpack'] = {'valid': False, 'exit_code': baked.returncode}
        passed = False

signature = subprocess.run(['codesign', '--verify', '--deep', '--strict', str(executable.parent.parent.parent)],
    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
passed = passed and signature.returncode == 0
report = {
    'schema': 'p4desk.macos-probes.v1', 'recorded_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'os_version': subprocess.check_output(['sw_vers', '-productVersion'], text=True).strip(),
    'os_build': subprocess.check_output(['sw_vers', '-buildVersion'], text=True).strip(),
    'architecture': platform.machine(), 'bundle_id': 'com.p4desk.mac',
    'executable_sha256': hashlib.sha256(executable.read_bytes()).hexdigest(),
    'codesign_verified': signature.returncode == 0, 'capture_requested': False,
    'screen_content_saved': False, 'usb_hardware_transfer_tested': False,
    'input_permission_requested': False, 'all_probes_passed': passed, 'results': results,
}
destination.parent.mkdir(parents=True, exist_ok=True)
temporary = destination.with_suffix(destination.suffix + '.tmp')
temporary.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
temporary.replace(destination)
print('Mac probe evidence:', destination)
sys.exit(0 if passed else 1)
PY
