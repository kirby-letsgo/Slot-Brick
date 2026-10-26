# Beetle PCE Fast (PC Engine and PC Engine CD)

Distributed as a separate libretro library under GPL-2.0-or-later. The exact
corresponding upstream source, including bundled dependencies, is:

https://github.com/libretro/beetle-pce-fast-libretro/tree/4eee99c451f85ebe66d1591de88b892015b23059

Archive: https://github.com/libretro/beetle-pce-fast-libretro/archive/4eee99c451f85ebe66d1591de88b892015b23059.tar.gz

SHA-256: `f15ec299283ff1022117508036ea8806782f120c40bf7757df042b6591c92613`

Slot applies `brick/pce-disc-control.patch` to expose playlist disc selection
through libretro and reject incomplete playlists without shifting disc indices.
The patch, checksum-pinned build recipe (`brick/build-pce.sh`), compiler setup
and packaging scripts are available at https://github.com/kirby-letsgo/Slot-Brick .
Use `sh brick/docker-build.sh` for the device or `sh brick/build-pce.sh host`
for a simulator core. CHD is enabled and CDs stream from disk.

The accompanying GPL and component notices cover Mednafen/Beetle PCE Fast,
libretro-common, libchdr (including dr_flac), LZMA 19.00, Zstandard, zlib and
Tremor. The source archive retains all original notices. No proprietary BIOS
or game ROMs are included. The simulator diagnostic is original test code.
