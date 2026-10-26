#!/bin/sh
set -eu
: "${BRICK_TOOLCHAIN:?Set BRICK_TOOLCHAIN}"
cd "$(dirname "$0")/.."
OUT="${1:-dist-brick}"
if [ -e "$OUT" ]; then echo "Output already exists: $OUT. Choose a new directory." >&2; exit 1; fi
mkdir -p "$OUT"
cp -R brick/card/. "$OUT/"

# Boot Logo app: picture converter plus its own copy of the file checker.
( cd brick/bootlogo-tool && cargo build --locked --release --target aarch64-unknown-linux-gnu )
cp brick/bootlogo-tool/target/aarch64-unknown-linux-gnu/release/slot-bootlogo "$OUT/Apps/BootLogo/"
cp brick/card/Slot/System/slot-filecheck "$OUT/Apps/BootLogo/"
cp target/aarch64-unknown-linux-gnu/device/slot "$OUT/Slot/System/"
cp target/pico8-sdl-nosensor.so "$OUT/Slot/System/"
cp vendor/mgba_libretro.so vendor/gpsp_libretro.so vendor/fceumm_libretro.so vendor/snes9x_libretro.so vendor/pcsx_rearmed_libretro.so vendor/genesis_plus_gx_libretro.so vendor/parallel_n64_libretro.so vendor/mednafen_ngp_libretro.so vendor/picodrive_libretro.so vendor/mednafen_pce_fast_libretro.so vendor/fbneo_neogeo_libretro.so "$OUT/Slot/System/"
for f in slot mgba_libretro.so gpsp_libretro.so fceumm_libretro.so snes9x_libretro.so pcsx_rearmed_libretro.so genesis_plus_gx_libretro.so parallel_n64_libretro.so mednafen_ngp_libretro.so picodrive_libretro.so mednafen_pce_fast_libretro.so fbneo_neogeo_libretro.so; do "$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-strip" "$OUT/Slot/System/$f"; done
for d in BIOS BIOS/PICO8 Config Config/PICO8 Games/GB Games/GBC Games/GBA Games/NES Games/SNES Games/MD Games/PS1 Games/GG Games/SMS Games/N64 Games/NGP Games/PICO8 Games/32X Games/SEGACD Games/PCE Games/PCECD Games/NEOGEO Labels/GB Labels/GBC Labels/GBA Labels/NES Labels/SNES Labels/MD Labels/PS1 Labels/GG Labels/SMS Labels/N64 Labels/NGP Labels/PICO8 Labels/32X Labels/SEGACD Labels/PCE Labels/PCECD Labels/NEOGEO Saves/GB Saves/GBC Saves/GBA Saves/NES Saves/SNES Saves/MD Saves/PS1 Saves/GG Saves/SMS Saves/N64 Saves/NGP Saves/PICO8 Saves/32X Saves/SEGACD Saves/PCE Saves/PCECD Saves/NEOGEO States/GB States/GBC States/GBA States/NES States/SNES States/MD States/PS1 States/GG States/SMS States/N64 States/NGP States/PICO8 States/32X States/SEGACD States/PCE States/PCECD States/NEOGEO Overlays/GB Overlays/GBC Overlays/GBA Overlays/NES Overlays/SNES Overlays/MD Overlays/PS1 Overlays/GG Overlays/SMS Overlays/N64 Overlays/NGP Overlays/PICO8 Overlays/32X Overlays/SEGACD Overlays/PCE Overlays/PCECD Overlays/NEOGEO Wallpapers Logs; do mkdir -p "$OUT/Slot/$d"; done
cp brick/README.md "$OUT/Slot/README.md"
# Overlays kept on this machine only (brick/overlays-local is not in git, so a card built by
# GitHub never has them): PNGs only, for a card built here for its own owner.
for d in GB GBC GBA NES SNES MD PS1 GG SMS N64 NGP 32X SEGACD PCE PCECD NEOGEO; do
    for f in brick/overlays-local/"$d"/*.png; do
        if [ -f "$f" ]; then cp "$f" "$OUT/Slot/Overlays/$d/"; fi
    done
done
cp -R licenses "$OUT/Slot/System/"
cp LICENSE "$OUT/Slot/System/licenses/slot-GPL-3.0.txt"
chmod +x "$OUT/Slot/System/pico8-bin/wget" "$OUT/Slot/System/slot" "$OUT/Slot/System/"*.sh "$OUT/Apps/"*/launch.sh "$OUT/Apps/BootLogo/slot-bootlogo" "$OUT/Apps/BootLogo/slot-filecheck"
echo "Card files written to $OUT"
