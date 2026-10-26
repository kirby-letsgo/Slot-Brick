#!/bin/sh
# Pinned Beetle Cygne and Beetle Lynx, for the card and native simulator.
set -eu
cd "$(dirname "$0")/.."
MODE=${1:-brick}
CACHE="$PWD/.docker-cache/core-archives"
mkdir -p "$CACHE"
if [ "$MODE" = brick ]; then
    : "${BRICK_TOOLCHAIN:?Set BRICK_TOOLCHAIN}"
    OUT="$PWD/vendor"
    CC="$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-gcc"
    CXX="$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-g++"
    AR="$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-ar"
else
    OUT="$PWD/target-sim/cores"; CC=cc; CXX=c++; AR=ar
fi
stamp=$(sha256sum "$0" | cut -d' ' -f1)
while read -r name repo rev checksum; do
    archive="$CACHE/$name-src.tar.gz"
    if [ ! -f "$archive" ]; then
        curl -fL --retry 3 "https://github.com/libretro/$repo/archive/$rev.tar.gz" -o "$archive.part"
        mv "$archive.part" "$archive"
    fi
    echo "$checksum  $archive" | sha256sum -c -
    if [ "$(cat "$OUT/.$name-stamp" 2>/dev/null || true)" = "$stamp" ] && [ -f "$OUT/${name}_libretro.so" ]; then continue; fi
    BUILD="$PWD/brick/core-build/$name-$MODE-$stamp"
    mkdir -p "$BUILD" "$OUT"
    tar --no-same-owner --strip-components=1 -xzf "$archive" -C "$BUILD"
    make -C "$BUILD" -j"${CORE_JOBS:-4}" platform=unix CC="$CC" CXX="$CXX" AR="$AR" \
        GIT_VERSION="$(printf '%s' "$rev" | cut -c1-7)" \
        LDFLAGS="-shared -Wl,--no-undefined -Wl,--version-script=link.T -static-libstdc++ -static-libgcc -lm -lrt"
    cp "$BUILD/${name}_libretro.so" "$OUT/"
    printf '%s\n' "$stamp" > "$OUT/.$name-stamp"
done <<'CORES'
mednafen_wswan beetle-wswan-libretro 4b01295838ea89e3f1355bbe4cb5cf98aa6108cd bce15c0e2505e15b7b55fa1d51b4a12219d07a67be62bbca5c26678d0d89c659
mednafen_lynx beetle-lynx-libretro fcdefcfb3c11d6d2e71be076a5d3df2e88ab73ed 3dd45a6f6585c444a415cb097996c572467754748e7163eca466d89b31bd8b37
CORES
