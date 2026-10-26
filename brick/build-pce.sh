#!/bin/sh
# Pinned Beetle PCE Fast for HuCards and CD-ROM², including CHD support.
set -eu
cd "$(dirname "$0")/.."
MODE=${1:-brick}
REV=4eee99c451f85ebe66d1591de88b892015b23059
CACHE="$PWD/.docker-cache/core-archives"
mkdir -p "$CACHE"
archive="$CACHE/mednafen_pce_fast-src.tar.gz"
if [ ! -f "$archive" ]; then
    curl -fL --retry 3 "https://github.com/libretro/beetle-pce-fast-libretro/archive/$REV.tar.gz" -o "$archive.part"
    mv "$archive.part" "$archive"
fi
echo "f15ec299283ff1022117508036ea8806782f120c40bf7757df042b6591c92613  $archive" | sha256sum -c -
stamp=$(cat "$0" brick/pce-disc-control.patch | sha256sum | cut -d' ' -f1)
if [ "$MODE" = brick ]; then
    : "${BRICK_TOOLCHAIN:?Set BRICK_TOOLCHAIN}"
    OUT="$PWD/vendor"
    CC="$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-gcc"
    CXX="$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-g++"
    AR="$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-ar"
else
    OUT="$PWD/target-sim/cores"; CC=cc; CXX=c++; AR=ar
fi
if [ "$(cat "$OUT/.pce-stamp" 2>/dev/null || true)" = "$stamp" ] && [ -f "$OUT/mednafen_pce_fast_libretro.so" ]; then exit 0; fi
BUILD="$PWD/brick/core-build/pce-$MODE-$stamp"
mkdir -p "$OUT" "$BUILD"
tar --no-same-owner --strip-components=1 -xzf "$archive" -C "$BUILD"
git -C "$BUILD" apply "$PWD/brick/pce-disc-control.patch"
# Stream CDs from disk instead of caching a whole disc in the Brick's memory.
make -C "$BUILD" -j"${CORE_JOBS:-4}" platform=unix CC="$CC" CXX="$CXX" AR="$AR" \
    HAVE_CHD=1 HAVE_CDROM=0 CACHE_CD=0 GIT_VERSION=4eee99c \
    LDFLAGS="-shared -Wl,--no-undefined -Wl,--version-script=link.T -static-libstdc++ -static-libgcc -lm"
cp "$BUILD/mednafen_pce_fast_libretro.so" "$OUT/"
printf '%s\n' "$stamp" > "$OUT/.pce-stamp"
