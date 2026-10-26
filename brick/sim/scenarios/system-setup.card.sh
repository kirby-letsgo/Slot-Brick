#!/bin/sh
set -eu
python3 - "$1" <<'PYCODE'
from pathlib import Path
import sys
root = Path(sys.argv[1])
# This scenario never launches emulators: tiny ELF headers represent installed files.
for path in (root/'System').glob('*_libretro.so'):
    path.unlink()
for name in ['mgba','picodrive','mednafen_pce_fast','fbneo_neogeo','pcsx_rearmed']:
    h=bytearray(64);h[:4]=b'\x7fELF';h[4:6]=bytes([2,1]);h[16]=3;h[18]=62
    (root/'System'/f'{name}_libretro.so').write_bytes(h)
(root/'BIOS/bios_CD_U.bin').write_bytes(bytes(128*1024))
PYCODE
