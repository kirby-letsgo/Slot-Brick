#!/usr/bin/env python3
"""Generate original diagnostic chips, never a commercial BIOS/game.

Uses the nam1975 driver solely for its public chip layout. FBNeo's patched-ROM
folder permits these replacement chips without requiring copyrighted CRCs.
Usage: make-neogeo.py SLOT_ROOT FBNEO_SOURCE_OR_TARBALL
"""
from pathlib import Path
import re
import struct
import sys
import tarfile
import zipfile
root, source = map(Path, sys.argv[1:3])
for d in ['Games/NEOGEO', 'BIOS/fbneo/patched', 'System', 'Saves/NEOGEO']:
    (root / d).mkdir(parents=True, exist_ok=True)
if source.is_file():
    with tarfile.open(source) as archive:
        driver = next(m for m in archive.getmembers() if m.name.endswith('/src/burn/drv/neogeo/d_neogeo.cpp'))
        text = archive.extractfile(driver).read().decode()
else:
    text = (source / 'src/burn/drv/neogeo/d_neogeo.cpp').read_text()
# 68000: disable interrupts, set palette entry 4095 green, feed watchdog, loop.
code = bytes.fromhex('46fc2700 33fc20f000401ffe 13fc000000300001 60f6')
for driver in ['nam1975', 'neogeo']:
    block = text.split(f'static struct BurnRomInfo {driver}RomDesc[] = {{', 1)[1].split('};', 1)[0]
    with zipfile.ZipFile(root / f'BIOS/fbneo/patched/{driver}.zip', 'w', zipfile.ZIP_DEFLATED) as z:
        for name, length, flags in re.findall(r'\{ "([^"\n]+)",\s*(0x[0-9a-fA-F]+),\s*0x[0-9a-fA-F]+,\s*([^}]+)', block):
            size = int(length, 16)
            data = bytearray(size)
            if 'BRF_PRG' in flags and ('BRF_BIOS' in flags and name != 'sm1.sm1' or name == '001-p1.p1'):
                data[:8] = struct.pack('>II', 0x10fffc, 0xc00100)
                data[0x100:0x100+len(code)] = code
                # Neo Geo P ROMs and BIOS chips store swapped 16-bit words.
                data[::2], data[1::2] = data[1::2], data[::2]
            elif name in ['sm1.sm1', '001-m1.m1']:
                data[:3] = bytes([0xf3, 0x18, 0xfe])  # Z80 DI; JR -2
            z.writestr(name, data)
    with zipfile.ZipFile(root / ('Games/NEOGEO/nam1975.zip' if driver == 'nam1975' else 'BIOS/neogeo.zip'), 'w') as z:
        z.writestr('slot-diagnostic.txt', 'Original test chips live in BIOS/fbneo/patched.')
