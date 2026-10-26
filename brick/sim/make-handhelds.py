#!/usr/bin/env python3
"""Original tiny diagnostic programs, with no commercial game or BIOS contents.
Usage: make-handhelds.py SLOT_ROOT
"""
from pathlib import Path
import sys
root = Path(sys.argv[1])
for d in ['Games/WS', 'Games/LYNX', 'BIOS']:
    (root/d).mkdir(parents=True, exist_ok=True)
for ext, color in [('ws', 0), ('wsc', 1)]:
    rom = bytearray(65536)
    # V30: CLI, enable LCD, set the background shade, loop forever.
    code = bytes([0xfa, 0xb0, 1, 0xe6, 0x14, 0xb0, 0, 0xe6, 0x01,
                  0xb0, 0x00, 0xe6, 0x1c, 0xeb, 0xfe])
    if color:
        # DS=0, enable color mode, set background palette entry 0 to green.
        code = bytes([0xfa, 0x31, 0xc0, 0x8e, 0xd8, 0xb0, 0x80, 0xe6, 0x60,
                      0xc7, 0x06, 0x00, 0xfe, 0xf0, 0x00,
                      0xb0, 1, 0xe6, 0x14, 0xb0, 0, 0xe6, 1, 0xeb, 0xfe])
    rom[:len(code)] = code
    rom[-16:-11] = bytes([0xea, 0, 0, 0, 0xf0])  # JMP F000:0000
    rom[-9] = color
    rom[-5] = 1  # 8 KiB battery SRAM
    rom[-2:] = (sum(rom[:-2]) & 65535).to_bytes(2, 'little')
    (root/f'Games/WS/{"Color" if color else "Mono"} Probe.{ext}').write_bytes(rom)
# Lynx BS93 homebrew starts directly in RAM and does not execute the boot ROM.
# BRA past header; SEI; make every palette entry green; loop forever.
code = bytearray([0x78, 0xa9, 15])
for n in range(16): code += bytes([0x8d, 0xa0+n, 0xfd])
code += bytes([0xa9, 0])
for n in range(16): code += bytes([0x8d, 0xb0+n, 0xfd])
pc = 0x20a + len(code)
code += bytes([0x4c, pc & 255, pc >> 8])
header = bytes([0x80, 8]) + (0x20a).to_bytes(2, 'big') + (len(code)+10).to_bytes(2, 'big') + b'BS93'
(root/'Games/LYNX/Lynx Probe.o').write_bytes(header+code)
# A test-only 512-byte placeholder. Never packaged or suitable for retail carts.
(root/'BIOS/lynxboot.img').write_bytes(bytes(512))
