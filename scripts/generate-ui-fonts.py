#!/usr/bin/env python3
"""Bake or reproduce the fixed, unmodified HarmonyOS Sans UI glyphs."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SIZES = [14, 16, 18, 22, 24, 26, 28, 34, 36]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    source = next(entry for entry in json.loads((ROOT / 'assets/fonts/SOURCES.json').read_text())
                  if entry['name'] == 'HarmonyOS Sans SC Regular')
    font, text = ROOT / source['path'], ROOT / 'assets/generated/ui-text.txt'
    if digest(font) != source['sha256'] or font.stat().st_size != source['bytes']:
        raise SystemExit('UI 字体与固定官方来源不一致。')
    subprocess.run(['cargo', 'build', '--release', '-p', 'p4desk-fontpack'], cwd=ROOT, check=True)
    helper = ROOT / 'target/release/p4desk-fontpack'
    with tempfile.TemporaryDirectory(prefix='p4desk-ui-font-') as directory:
        output = Path(directory) / 'ui.p4f'
        result = subprocess.run([str(helper), 'bake', '--font', str(font), '--text-file', str(text),
                                 '--output', str(output), '--sizes', ','.join(map(str, SIZES))],
                                cwd=ROOT, check=True, capture_output=True, text=True)
        baked = json.loads(result.stdout)
        subprocess.run([str(helper), 'validate', '--input', str(output)], check=True,
                       capture_output=True)
        metadata = {
            'schema': 'p4desk.ui-typeface.v1', 'family': source['name'],
            'source_path': source['path'], 'source_sha256': source['sha256'],
            'source_commit': source['commit'], 'license': source['license'],
            'outlines_modified': False, 'rasterizer': 'fontdue 0.9.4',
            'pixel_format': 'alpha8', 'sizes': SIZES, 'glyph_count': baked['glyph_count'],
            'text_path': 'assets/generated/ui-text.txt', 'text_sha256': digest(text),
            'bytes': output.stat().st_size, 'sha256': digest(output),
        }
        encoded = (json.dumps(metadata, ensure_ascii=False, indent=2) + '\n').encode()
        target, manifest = ROOT / 'assets/generated/ui.p4f', ROOT / 'assets/generated/ui-font.json'
        if args.check:
            if output.read_bytes() != target.read_bytes() or encoded != manifest.read_bytes():
                raise SystemExit('UI 字形与已保存资产不一致。')
        else:
            target.write_bytes(output.read_bytes())
            manifest.write_bytes(encoded)
        print(json.dumps({'valid': True, 'mode': 'check' if args.check else 'generate',
                          'bytes': metadata['bytes'], 'glyph_count': metadata['glyph_count'],
                          'sizes': SIZES}, ensure_ascii=False))


if __name__ == '__main__':
    main()
