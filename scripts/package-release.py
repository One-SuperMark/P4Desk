#!/usr/bin/env python3
"""Package a committed source tree, signed Mac app, firmware and recovery tools."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]
OWNER = 'p4desk.release.v1'


def digest(path):
    sha256 = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            sha256.update(chunk)
    return sha256.hexdigest()


def copy_required(source, destination):
    if not source.is_file() or source.stat().st_size == 0:
        raise SystemExit(f'交付文件缺失或为空：{source}')
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build-dir', type=Path, default=Path('/Volumes/work/esp/build/p4desk'))
    parser.add_argument('--app', type=Path, default=ROOT / 'dist/P4Desk.app')
    parser.add_argument('--output-dir', type=Path, default=ROOT / 'dist')
    parser.add_argument('--version', default='0.1.0')
    args = parser.parse_args()
    if not re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+(?:-[a-z0-9.]+)?', args.version):
        parser.error('版本号须为 0.1.0 或 0.1.0-preview.1 格式')
    build, app, output = args.build_dir.resolve(), args.app.resolve(), args.output_dir.resolve()
    commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    dirty = subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True)
    if dirty:
        raise SystemExit('先提交源文件和验收记录，使源码归档与交付清单对应同一个 Git 提交。')
    subprocess.run(['codesign', '--verify', '--deep', '--strict', str(app)], check=True)
    for relative in ['Contents/MacOS/P4Desk', 'Contents/Resources/p4desk-fontpack',
                     'Contents/Resources/HarmonyOS_Sans_SC_Regular.ttf',
                     'Contents/Resources/HarmonyOS-Sans-LICENSE.txt', 'Contents/Resources/FONT-SOURCES.json']:
        resource = app / relative
        if not resource.is_file() or resource.stat().st_size == 0:
            raise SystemExit(f'Mac app 必需资源缺失：{relative}')
    metadata = json.loads((build / 'flasher_args.json').read_text())
    output.mkdir(parents=True, exist_ok=True)
    name = f'P4Desk-{args.version}'
    destination, archive = output / name, output / f'{name}.zip'
    if destination.exists():
        marker = destination / '.p4desk-release'
        if destination.is_symlink() or not marker.is_file() or marker.read_text().strip() != OWNER:
            raise SystemExit(f'目标目录不是本脚本生成的交付目录：{destination}')
    with tempfile.TemporaryDirectory(prefix='.p4desk-package-', dir=output) as temporary:
        package = Path(temporary) / name
        package.mkdir()
        (package / '.p4desk-release').write_text(OWNER + '\n')
        shutil.copytree(app, package / 'P4Desk.app', symlinks=True)
        for offset, relative in metadata['flash_files'].items():
            binary = (build / relative).resolve()
            if not binary.is_relative_to(build) or not 0x2000 <= int(offset, 0) < 32 * 1024 * 1024:
                raise SystemExit('构建清单中的固件路径或地址无效')
            copy_required(binary, package / 'firmware' / relative)
        copy_required(build / 'p4desk.elf', package / 'firmware/p4desk.elf')
        copy_required(build / 'flasher_args.json', package / 'firmware/flasher_args.json')
        copy_required(ROOT / 'firmware/partitions.csv', package / 'firmware/partitions.csv')
        copy_required(ROOT / 'firmware/THIRD-PARTY.md', package / 'firmware/THIRD-PARTY.md')
        copy_required(ROOT / 'README.md', package / 'README.md')
        copy_required(ROOT / 'LICENSE', package / 'LICENSE')
        shutil.copytree(ROOT / 'docs', package / 'docs')
        shutil.copytree(ROOT / 'third_party/licenses', package / 'third_party/licenses')
        copy_required(ROOT / 'desktop/macos/README.md', package / 'desktop/macos/README.md')
        copy_required(ROOT / 'desktop/macos/THIRD_PARTY_NOTICES.md', package / 'desktop/macos/THIRD_PARTY_NOTICES.md')
        shutil.copytree(ROOT / 'desktop/macos/acceptance', package / 'desktop/macos/acceptance')
        for script in ['backup-device.py', 'device-tool.py']:
            copy_required(ROOT / 'scripts' / script, package / 'scripts' / script)
        # Font files stay bundled with P4Desk rather than distributed as a font-only tool.
        for relative in ['p4desk-fontpack', 'FONT-SOURCES.json']:
            copy_required(app / 'Contents/Resources' / relative, package / 'tools/fontpack' / relative)
        copy_required(ROOT / 'tools/fontpack/README.md', package / 'tools/fontpack/README.md')
        source = package / f'{name}-source.tar.gz'
        subprocess.run(['git', 'archive', '--format=tar.gz', f'--prefix={name}-source/',
                        '--output', str(source), commit], cwd=ROOT, check=True)
        files = {}
        for path in sorted(package.rglob('*')):
            if path.is_file():
                files[path.relative_to(package).as_posix()] = {
                    'bytes': path.stat().st_size, 'sha256': digest(path)}
        manifest = {
            'schema': OWNER, 'project': 'P4Desk', 'version': args.version,
            'recorded_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'source_commit': commit, 'toolchains': {
                'esp_idf': '6.0.2', 'rust': 'nightly-2026-09-27',
                'firmware_target': 'riscv32imafc-esp-espidf', 'mac_architecture': 'arm64'},
            'mac_signature_verified': True,
            'acceptance_record': 'docs/acceptance.md',
            'backups_included': False, 'files': files,
        }
        (package / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
        subprocess.run(['codesign', '--verify', '--deep', '--strict', str(package / 'P4Desk.app')], check=True)
        temporary_archive = Path(temporary) / f'{name}.zip'
        subprocess.run(['ditto', '-c', '-k', '--sequesterRsrc', '--keepParent',
                        str(package), str(temporary_archive)], check=True)
        if destination.exists():
            shutil.rmtree(destination)
        package.replace(destination)
        temporary_archive.replace(archive)
    print('交付目录：', destination)
    print('交付 ZIP：', archive)
    print('ZIP SHA256：', digest(archive))
    print('对应源码提交：', commit)


if __name__ == '__main__':
    main()
