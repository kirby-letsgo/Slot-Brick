#!/bin/sh
# PicoDrive for 32X and Sega CD, pinned with its libretro build dependencies.
set -eu
cd "$(dirname "$0")/.."
MODE=${1:-brick}
CACHE="$PWD/.docker-cache/core-archives"
mkdir -p "$CACHE"
fetch() {
    file="$CACHE/$1-src.tar.gz"
    if [ ! -f "$file" ]; then
        curl -fL --retry 3 "$2" -o "$file.part"
        mv "$file.part" "$file"
    fi
    echo "$3  $file" | sha256sum -c -
}
fetch picodrive https://github.com/libretro/picodrive/archive/1890c2932234c9d30f4cd3851d02228baae8f09e.tar.gz 123871ce468bc31f824df9273415215527ded2bdaadffd0503a97149aa06d523
fetch picodrive-libpicofe https://github.com/irixxxx/libpicofe/archive/9ed5822606dd7ff20a782a882e8fd611cb53ba88.tar.gz 458519aaa70916528e593f1f052aa45f704670fba094b409478a8197e6f076a7
fetch picodrive-emu2413 https://github.com/digital-sound-antiques/emu2413/archive/a2dfc20ff507e4fd075cd325620bcea655e2c1f7.tar.gz bd816079cfbf13c56a260c095d6b8e260c2f877e5b13f50d4e570b96e8b92830
fetch picodrive-dr_libs https://github.com/mackron/dr_libs/archive/dd762b861ecadf5ddd5fb03e9ca1db6707b54fbb.tar.gz 8f0be8a4aa27cd4ddaba41acbb845b1641964f2cc2f5a5f7b4b72f73368c1c12
fetch picodrive-libchdr https://github.com/irixxxx/libchdr/archive/e62ac5995b1c7ef65ece35293914843b8ee57d49.tar.gz 59b99005d3d6317a0d1e45e1c4abf0e551070dbf32c8fdc66ed5b4ae6d8ea702
stamp=$(sha256sum "$0" | cut -d' ' -f1)
if [ "$MODE" = brick ]; then
    : "${BRICK_TOOLCHAIN:?Set BRICK_TOOLCHAIN}"
    OUT="$PWD/vendor"; PLATFORM=aarch64
    CC="$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-gcc"
    AR="$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-ar"
else
    OUT="$PWD/target-sim/cores"; PLATFORM=unix; CC=cc; AR=ar
fi
if [ "$(cat "$OUT/.picodrive-stamp" 2>/dev/null || true)" = "$stamp" ] && [ -f "$OUT/picodrive_libretro.so" ]; then exit 0; fi
BUILD="$PWD/brick/core-build/picodrive-$MODE-$stamp"
unpack() {
    mkdir -p "$BUILD/$2"
    tar --no-same-owner --strip-components=1 -xzf "$CACHE/$1-src.tar.gz" -C "$BUILD/$2"
}
mkdir -p "$OUT" "$BUILD"
unpack picodrive .
unpack picodrive-libpicofe platform/libpicofe
unpack picodrive-emu2413 pico/sound/emu2413
unpack picodrive-dr_libs platform/common/dr_libs
unpack picodrive-libchdr pico/cd/libchdr
# CHD includes bundled LZMA/Zstd decoders; SH-2 dynarec stays enabled for 32X.
# Sega CD BIOS files are supplied by the player, never packaged.
make -C "$BUILD" -f Makefile.libretro -j"${CORE_JOBS:-4}" \
    platform="$PLATFORM" CC="$CC" AR="$AR" use_libchdr=1 GIT_REVISION=-1890c29
cp "$BUILD/picodrive_libretro.so" "$OUT/"
printf '%s\n' "$stamp" > "$OUT/.picodrive-stamp"
