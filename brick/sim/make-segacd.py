"""Synthetic Sega CD BIOS and data discs for integration checks, written from scratch.

This is a tiny VDP test program, NOT a replacement BIOS for playing games.
Usage: python3 make-segacd.py SLOT_ROOT
"""
from pathlib import Path
import struct
import sys

root = Path(sys.argv[1])
(root / 'BIOS').mkdir(parents=True, exist_ok=True)
games = root / 'Games/SEGACD'
games.mkdir(parents=True, exist_ok=True)
bios = bytearray(128 * 1024)
bios[:8] = struct.pack('>II', 0x00fffe00, 0x200)
bios[0x100:0x110] = b'SEGA MEGA DRIVE '
program = [
    0x46fc, 0x2700,          # mask interrupts
    0x41f9, 0x00c0, 0x0004, # VDP control
    0x43f9, 0x00c0, 0x0000, # VDP data
    0x30bc, 0x8004, 0x30bc, 0x8104,
    0x30bc, 0x8c81, 0x30bc, 0x8f02, 0x30bc, 0x8700,
    0x20bc, 0xc000, 0x0000,
    0x32bc, 0x00e0,          # green backdrop
    0x30bc, 0x8144, 0x60fe,
]
bios[0x200:0x200 + 2 * len(program)] = struct.pack(f'>{len(program)}H', *program)
for region in 'UEJ':
    (root / f'BIOS/bios_CD_{region}.bin').write_bytes(bios)
for n in [1, 2]:
    data = bytearray(2048 * 300)
    data[:14] = b'SEGADISCSYSTEM\0'
    data[0x100:0x110] = b'SEGA MEGA DRIVE '
    data[0x120:0x130] = b'SLOT CD TEST    '
    (games / f'CD Test (Disc {n}).iso').write_bytes(data)
    (games / f'CD Test (Disc {n}).cue').write_text(
        f'FILE "CD Test (Disc {n}).iso" BINARY\n  TRACK 01 MODE1/2048\n    INDEX 01 00:00:00\n')
(games / 'CD Test.m3u').write_text('CD Test (Disc 1).cue\nCD Test (Disc 2).cue\n')
