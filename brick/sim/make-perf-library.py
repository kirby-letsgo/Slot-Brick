#!/usr/bin/env python3
"""Generate header-only ZIP fixtures for library profiling; these are not playable ROMs."""
import argparse
import struct
import zipfile
import zlib
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('root', type=Path, help='new Slot folder (must not already exist)')
parser.add_argument('--per-system', type=int, default=1000)
parser.add_argument('--labels', type=int, default=0, help='number of Game Boy PNG labels')
args = parser.parse_args()
if args.per_system < 1 or not 0 <= args.labels <= args.per_system:
    parser.error('per-system must be positive; labels must be between 0 and per-system')
if args.root.exists():
    parser.error('destination already exists; choose a new fixture folder')

def chunk(kind, data):
    return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))

for system, ext in [('GB', 'gb'), ('NES', 'nes'), ('SNES', 'sfc'), ('MD', 'md')]:
    folder = args.root / 'Games' / system
    folder.mkdir(parents=True)
    data = bytearray(65536)
    if system == 'GB':
        data[0x143] = 0x80
    elif system == 'NES':
        data[:4] = b'NES\x1a'
    elif system == 'SNES':
        data[0x7fd5] = 0x20
        data[0x7fd9] = 2
        data[0x7fdc:0x7fe0] = b'\xff\xff\x00\x00'
    elif system == 'MD':
        data[0x100:0x104] = b'SEGA'
        data[0x1f0:0x1f3] = b'JUE'
    for i in range(args.per_system):
        with zipfile.ZipFile(folder / f'Game {i:04}.zip', 'w', zipfile.ZIP_DEFLATED) as archive:
            archive.writestr(f'Game {i:04}.{ext}', data)

if args.labels:
    folder = args.root / 'Labels/GB'
    folder.mkdir(parents=True)
    w, h = 512, 512
    rows = (b'\0' + bytes([160, 90, 40]) * w) * h
    png = (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0))
           + chunk(b'IDAT', zlib.compress(rows)) + chunk(b'IEND', b''))
    for i in range(args.labels):
        (folder / f'Game {i:04}.png').write_bytes(png)
print(f'{args.per_system * 4} synthetic ZIPs, {args.labels} labels: {args.root}')
