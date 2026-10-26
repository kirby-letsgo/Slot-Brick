#!/bin/sh
set -eu
root="$1"
for cart in GG/Sample.gg SMS/Sample.sms N64/Sample.z64 NGP/Sample.ngp PICO8/Sample.p8; do
    printf 'simulator stand-in cartridge\n' > "$root/Games/$cart"
done
mkdir -p "$root/Wallpapers"
# A generated test image exercises the real wallpaper loader without shipping artwork.
python3 - "$root/Wallpapers/Grid-test.png" <<'PY'
import struct, sys, zlib
w, h = 640, 480
rows = bytearray()
for y in range(h):
    rows.append(0)
    for x in range(w):
        rows.extend((76, 108, 132) if (x // 80 + y // 80) % 2 else (102, 132, 154))
def chunk(kind, data):
    return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))
with open(sys.argv[1], 'wb') as f:
    f.write(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(rows)) + chunk(b'IEND', b''))
PY
