#!/usr/bin/env python3
"""Read a private full-flash backup in MD5-checked, bounded chunks. Never flash."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

FLASH_BYTES = 32 * 1024 * 1024
CHUNK_BYTES = 1024 * 1024


def worker(args):
    import esptool
    out = Path(args.output).resolve()
    parts = out.parent / (out.stem + '.parts')
    parts.mkdir(mode=0o700, exist_ok=True)
    os.chmod(parts, 0o700)
    esp = None
    records = []
    started = time.monotonic()
    try:
        for offset in range(0, FLASH_BYTES, CHUNK_BYTES):
            part = parts / f'{offset:08x}.bin'
            for attempt in range(3):
                try:
                    if esp is None:
                        esp = esptool.detect_chip(args.port, baud=115200)
                        if esp.CHIP_NAME != 'ESP32-P4':
                            raise RuntimeError('Attached device is not ESP32-P4')
                        esp = esptool.run_stub(esp)
                        rate = int(args.baud) if attempt == 0 else 115200
                        if rate != 115200:
                            esp.change_baud(rate)
                        esptool.attach_flash(esp)
                    # ESPLoader.read_flash verifies the device's MD5 for each chunk.
                    data = esp.read_flash(offset, CHUNK_BYTES)
                    if len(data) != CHUNK_BYTES:
                        raise RuntimeError('Unexpected flash chunk length')
                    with part.open('wb') as handle:
                        handle.write(data)
                        handle.flush()
                        os.fsync(handle.fileno())
                    records.append({'offset':offset, 'bytes':len(data),
                                    'sha256':hashlib.sha256(data).hexdigest()})
                    print(f'Flash backup: {(offset + CHUNK_BYTES) * 100 // FLASH_BYTES}%', flush=True)
                    break
                except Exception as error:
                    if esp is not None:
                        try:
                            esp._port.close()
                        except Exception:
                            pass
                        esp = None
                    print(f'Chunk {offset:#x} attempt {attempt + 1} failed: {type(error).__name__}', flush=True)
                    if attempt == 2:
                        raise
            else:
                raise RuntimeError('Flash chunk could not be backed up')
        temporary = out.with_suffix('.partial')
        digest = hashlib.sha256()
        with temporary.open('wb') as destination:
            for record in records:
                data = (parts / f'{record["offset"]:08x}.bin').read_bytes()
                if hashlib.sha256(data).hexdigest() != record['sha256']:
                    raise RuntimeError('Private chunk verification failed')
                destination.write(data)
                digest.update(data)
            destination.flush()
            os.fsync(destination.fileno())
        if temporary.stat().st_size != FLASH_BYTES:
            raise RuntimeError('Full backup size verification failed')
        os.replace(temporary, out)
        manifest = {'model':'ESP32-P4-WIFI6-Touch-LCD-7B', 'bytes':FLASH_BYTES,
                    'sha256':digest.hexdigest(), 'elapsed_seconds':round(time.monotonic()-started, 2),
                    'status':'device_md5_and_file_sha256_verified', 'chunks':records}
        out.with_suffix('.json').write_text(json.dumps(manifest, indent=2)+'\n')
        # Keep an independent NVS recovery file in the private backup directory.
        with out.open('rb') as handle:
            handle.seek(0x9000)
            (out.parent / (out.stem + '-nvs.bin')).write_bytes(handle.read(0x6000))
        print(json.dumps({key:value for key,value in manifest.items() if key != 'chunks'}), flush=True)
    finally:
        if esp is not None:
            try:
                esp.hard_reset()
            finally:
                esp._port.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--port', required=True)
    parser.add_argument('--python')
    parser.add_argument('--output', required=True)
    parser.add_argument('--baud', default='460800')
    parser.add_argument('--worker', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    out = Path(args.output).resolve()
    out.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    os.chmod(out.parent, 0o700)
    os.umask(0o077)
    if out.exists():
        raise SystemExit('Backup already exists; use a new output filename')
    if args.worker:
        worker(args)
        return
    if not args.python:
        parser.error('--python is required (ESP-IDF Python environment)')
    command = [args.python, str(Path(__file__).resolve()), '--worker', '--port', args.port,
               '--baud', args.baud, '--output', str(out)]
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1)
    for line in process.stdout:
        if re.search(r'MAC:|Reading MAC|Serial number', line, re.I):
            continue
        print(line.rstrip().replace(args.port, '<P4 serial port>'), flush=True)
    raise SystemExit(process.wait())


if __name__ == '__main__':
    main()
