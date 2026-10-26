# PicoDrive (32X)

PicoDrive is distributed as a separate libretro shared library. The build uses
unmodified upstream sources, FAME/CZ80 CPU emulation and the SH-2 dynamic
recompiler. The non-commercial terms are in picodrive-LICENSE.txt; other
component notices accompany it. No game ROMs or proprietary BIOS are included.

Exact sources (each GitHub tree also has an archive/REVISION.tar.gz download):

- https://github.com/libretro/picodrive/tree/1890c2932234c9d30f4cd3851d02228baae8f09e
- https://github.com/irixxxx/libpicofe/tree/9ed5822606dd7ff20a782a882e8fd611cb53ba88 (platform/libpicofe)
- https://github.com/digital-sound-antiques/emu2413/tree/a2dfc20ff507e4fd075cd325620bcea655e2c1f7 (pico/sound/emu2413)
- https://github.com/mackron/dr_libs/tree/dd762b861ecadf5ddd5fb03e9ca1db6707b54fbb (platform/common/dr_libs)

The build recipe, pinned archive SHA-256 checksums and compiler selection are in
brick/build-picodrive.sh at https://github.com/kirby-letsgo/Slot-Brick .
Build with sh brick/docker-build.sh, or sh brick/build-picodrive.sh host for a
native simulator core. CHD support is disabled; Slot exposes only 32X games
through this core. Mega Drive continues to use Genesis Plus GX.

PicoDrive includes work by Dave, fdave, notaz, irixxxx and contributors; FAME by
Oscar Orallo Pelaez (based on C68K by Stéphane Dallongeville), CZ80 by Stéphane
Dallongeville and NJ, MAME SH-2 emulation, libretro-common, zlib and Tremor.
The original source files retain the full component notices. libpicofe offers
MAME, GPL-2.0-or-later and LGPL-2.1-or-later terms; this build uses its platform
header under the MAME option.
