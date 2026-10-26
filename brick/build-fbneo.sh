#!/bin/sh
# Build only Neo Geo drivers; ROM archives remain intact for FBNeo to read.
set -eu
cd "$(dirname "$0")/.."
MODE=${1:-brick}
REV=32958a89abd645ed20387c8d31d21ede3bc48161
CACHE="$PWD/.docker-cache/core-archives"
mkdir -p "$CACHE"
archive="$CACHE/fbneo-src.tar.gz"
if [ ! -f "$archive" ]; then
    curl -fL --retry 3 "https://github.com/libretro/FBNeo/archive/$REV.tar.gz" -o "$archive.part"
    mv "$archive.part" "$archive"
fi
echo "29fafcf396b1ef0bf6d3d867f436cac3b50faef613bc0041619c71b2bad660f3  $archive" | sha256sum -c -
stamp=$(cat "$0" brick/fbneo-slot.patch | sha256sum | cut -d' ' -f1)
if [ "$MODE" = brick ]; then
    : "${BRICK_TOOLCHAIN:?Set BRICK_TOOLCHAIN}"
    OUT="$PWD/vendor"
    CC="$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-gcc"
    CXX="$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-g++"
    AR="$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-ar"
else
    OUT="$PWD/target-sim/cores"; CC=cc; CXX=c++; AR=ar
fi
if [ "$(cat "$OUT/.fbneo-stamp" 2>/dev/null || true)" = "$stamp" ] && [ -f "$OUT/fbneo_neogeo_libretro.so" ]; then exit 0; fi
BUILD="$PWD/brick/core-build/fbneo-$MODE-$stamp"
mkdir -p "$OUT" "$BUILD"
tar --no-same-owner --strip-components=1 -xzf "$archive" -C "$BUILD"
git -C "$BUILD" apply "$PWD/brick/fbneo-slot.patch"
# Generate the subset driver table first (also works with GNU make before .WAIT).
make -C "$BUILD/src/burner/libretro" platform=unix SUBSET=neogeo generate-files
make -C "$BUILD/src/burner/libretro" -j"${CORE_JOBS:-4}" platform=unix SUBSET=neogeo \
    CC="$CC" CXX="$CXX" LD="$CXX" AR="$AR" \
    INCLUDE_CHD_SUPPORT=1 INCLUDE_7Z_SUPPORT=1 USE_X64_DRC=0 \
    LDFLAGS="-lpthread -static-libstdc++ -static-libgcc" \
    GIT_VERSION=32958a8
cp "$BUILD/src/burner/libretro/fbneo_neogeo_libretro.so" "$OUT/"
printf '%s\n' "$stamp" > "$OUT/.fbneo-stamp"
