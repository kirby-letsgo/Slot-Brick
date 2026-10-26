#!/usr/bin/env python3
"""Generate an original HuC6280 diagnostic and synthetic CD firmware (no game/BIOS data).

The program selects a green backdrop and loops with interrupts disabled. The CD
fixture exercises loading, playlists and save state plumbing, not a commercial BIOS.
"""
import sys
from pathlib import Path

root = Path(sys.argv[1])
for d in ['Games/PCE', 'Games/PCECD', 'BIOS', 'System']:
    (root / d).mkdir(parents=True, exist_ok=True)
code = bytearray([0x78, 0xd8, 0xd4])  # SEI, CLD, CSH
# VCE palette address 0: 9-bit green backdrop (GGGRRRBBB).
for address, value in [(0x402, 0), (0x403, 0), (0x404, 0xc0), (0x405, 1)]:
    code += bytes([0xa9, value, 0x8d, address & 255, address >> 8])
# Also set sprite/backdrop entry 256, used when the background is disabled.
for address, value in [(0x402, 0), (0x403, 1), (0x404, 0xc0), (0x405, 1)]:
    code += bytes([0xa9, value, 0x8d, address & 255, address >> 8])
# Standard 256x240 display timing, background/sprites disabled.
for reg, value in [(5, 0), (9, 0), (10, 0x0202), (11, 0x041f), (12, 0x0f02), (13, 239), (14, 3)]:
    code += bytes([0x03, reg, 0x13, value & 255, 0x23, value >> 8])
loop = 0xe000 + len(code)
code += bytes([0x4c, loop & 255, loop >> 8])
rom = bytearray(8192)
rom[:len(code)] = code
for vector in range(0x1ff6, 0x2000, 2):
    rom[vector:vector+2] = bytes([0, 0xe0])
(root / 'Games/PCE/HuCard Test.pce').write_bytes(rom)
(root / 'BIOS/syscard3.pce').write_bytes(rom * 32)
cd = root / 'Games/PCECD'
# Enough sectors for a legal data track. No executable is read by this test firmware.
for n in [1, 2]:
    stem = f'CD Test (Disc {n})'
    (cd / f'{stem}.bin').write_bytes(bytes(2048 * 300))
    (cd / f'{stem}.cue').write_text(f'FILE "{stem}.bin" BINARY\n  TRACK 01 MODE1/2048\n    INDEX 01 00:00:00\n')
(cd / 'CD Test.m3u').write_text('CD Test (Disc 1).cue\nCD Test (Disc 2).cue\n')
