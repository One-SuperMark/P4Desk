#!/usr/bin/env python3
"""Flash only after a verified backup, or read a bounded, sanitized UART log."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import time


def clean(line, port):
    if re.search(r'MAC:|Reading MAC|Serial number', line, re.I):
        return None
    return line.replace(port, '<P4 serial port>').rstrip()


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open('rb') as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def run_tool(arguments, python, port):
    process = subprocess.Popen([python, '-m', 'esptool', '--chip', 'esp32p4',
                                '--port', port, *arguments], stdout=subprocess.PIPE,
                               stderr=subprocess.STDOUT, text=True)
    for line in process.stdout:
        safe = clean(line, port)
        if safe:
            print(safe, flush=True)
    if process.wait():
        raise SystemExit('esptool 操作失败，未标记为成功')


def flash(args):
    backup = Path(args.backup).resolve()
    manifest = json.loads(backup.with_suffix('.json').read_text())
    if manifest.get('status') != 'device_md5_and_file_sha256_verified' or backup.stat().st_size != 32 * 1024 * 1024:
        raise SystemExit('需要完整且校验通过的 32 MiB 备份；请先执行 backup-device.py')
    digest = sha256_file(backup)
    if digest != manifest.get('sha256'):
        raise SystemExit('备份 SHA256 不匹配，停止刷写')
    build = Path(args.build_dir).resolve()
    metadata = json.loads((build / 'flasher_args.json').read_text())
    pairs = []
    for offset, relative in sorted(metadata['flash_files'].items(), key=lambda item: int(item[0], 0)):
        binary = (build / relative).resolve()
        if not binary.is_relative_to(build) or not binary.is_file():
            raise SystemExit('固件构建产物不完整')
        address = int(offset, 0)
        if address < 0x2000 or address + binary.stat().st_size > 32 * 1024 * 1024:
            raise SystemExit('固件地址超出 P4Desk 允许范围')
        pairs.extend([offset, str(binary)])
    print('完整 Flash/NVS 备份已校验，开始写入构建清单列出的固件分区。', flush=True)
    run_tool(['--baud', args.baud, 'write-flash', *metadata['write_flash_args'], *pairs], args.python, args.port)


def monitor(args):
    # Run in the selected IDF environment, which already contains pyserial.
    if not args.worker:
        command = [args.python, str(Path(__file__).resolve()), 'monitor', '--worker',
                   '--port', args.port, '--duration', str(args.duration)]
        if args.reset:
            command.append('--reset')
        if args.output:
            command.extend(['--output', args.output])
        raise SystemExit(subprocess.call(command))
    import serial
    destination = None
    if args.output:
        path = Path(args.output).resolve()
        path.parent.mkdir(parents=True, exist_ok=True)
        destination = path.open('w')
    try:
        with serial.Serial(args.port, 115200, timeout=0.5) as uart:
            uart.dtr = False
            uart.rts = False
            if args.reset:
                uart.rts = True
                time.sleep(0.1)
                uart.rts = False
            deadline = time.monotonic() + args.duration
            while time.monotonic() < deadline:
                raw = uart.readline()
                if not raw:
                    continue
                safe = clean(raw.decode('utf-8', errors='replace'), args.port)
                if safe:
                    print(safe, flush=True)
                    if destination:
                        destination.write(safe + '\n')
                        destination.flush()
    finally:
        if destination:
            destination.close()


def main():
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest='operation', required=True)
    flashing = commands.add_parser('flash')
    flashing.add_argument('--port', required=True)
    flashing.add_argument('--python', required=True)
    flashing.add_argument('--backup', required=True)
    flashing.add_argument('--build-dir', default='/Volumes/work/esp/build/p4desk')
    flashing.add_argument('--baud', default='460800')
    logging = commands.add_parser('monitor')
    logging.add_argument('--port', required=True)
    logging.add_argument('--python')
    logging.add_argument('--duration', type=int, default=60)
    logging.add_argument('--reset', action='store_true')
    logging.add_argument('--output')
    logging.add_argument('--worker', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.operation == 'monitor':
        if not 1 <= args.duration <= 3600:
            parser.error('--duration must be between 1 and 3600 seconds')
        if not args.worker and not args.python:
            parser.error('--python is required')
        monitor(args)
    else:
        flash(args)


if __name__ == '__main__':
    main()
