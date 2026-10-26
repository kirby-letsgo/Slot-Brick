#!/bin/sh
# Pinned N64 and Neo Geo Pocket cores. Called by card and simulator builds.
# Sources are fetched at build time, verified, and cached outside the distributable card.
set -eu
cd "$(dirname "$0")/.."
MODE=${1:-brick}
N64=38befe1e6048de4abc8a9ba7c4ddeadaf342cdfc
NGP=a50d5ac288a81f2104ddf43195a4efdd15c72227
CACHE="$PWD/.docker-cache/core-archives"
mkdir -p "$CACHE"
fetch() {
    file="$CACHE/$1-src.tar.gz"
    if [ ! -f "$file" ]; then curl -fL --retry 3 "$2" -o "$file.part"; mv "$file.part" "$file"; fi
    echo "$3  $file" | sha256sum -c -
}
fetch parallel_n64 "https://github.com/libretro/parallel-n64/archive/$N64.tar.gz" 7ee9104e095b90bad08442c062610e1b9f749bb5f325b9e4a4555289b6a9b76b
fetch mednafen_ngp "https://github.com/libretro/beetle-ngp-libretro/archive/$NGP.tar.gz" ffeabf2a357548f3a55423d542c31bb1e08571d81d7104f8213df3bfa2eb9c9f
stamp=$(cat "$0" brick/n64-gles.patch | sha256sum | cut -d' ' -f1)
if [ "$MODE" = brick ]; then
    : "${BRICK_TOOLCHAIN:?Set BRICK_TOOLCHAIN}"
    export PATH="$BRICK_TOOLCHAIN/bin:$PATH"
    OUT="$PWD/vendor"; ARCH=aarch64; CC=aarch64-linux-gnu-gcc; CXX=aarch64-linux-gnu-g++; AR=aarch64-linux-gnu-ar
else
    OUT="$PWD/target-sim/cores"; ARCH=$(uname -m); CC=cc; CXX=c++; AR=ar
fi
BUILD="$PWD/brick/core-build/extra-$MODE-$stamp"
mkdir -p "$OUT" "$BUILD"
if [ "$(cat "$OUT/.extra-cores-stamp" 2>/dev/null || true)" = "$stamp" ] && [ -f "$OUT/parallel_n64_libretro.so" ] && [ -f "$OUT/mednafen_ngp_libretro.so" ]; then exit 0; fi
if [ ! -d "$BUILD/parallel-n64-$N64" ]; then
    tar --no-same-owner -xzf "$CACHE/parallel_n64-src.tar.gz" -C "$BUILD"
    git -C "$BUILD/parallel-n64-$N64" apply "$PWD/brick/n64-gles.patch"
fi
[ -d "$BUILD/beetle-ngp-libretro-$NGP" ] || tar --no-same-owner -xzf "$CACHE/mednafen_ngp-src.tar.gz" -C "$BUILD"
CPUFLAGS=""
GL_LIB=-lGLESv2
if [ "$MODE" = brick ]; then
    # TrimUI's toolchain lacks GLES headers and link libraries. Use Khronos headers from
    # the build box and a link-only stub with the firmware driver's SONAME. The stub is
    # NEVER packaged: all GLES calls resolve to the real PowerVR driver on the Brick.
    mkdir -p "$BUILD/include" "$BUILD/link"
    cp -R /usr/include/GLES2 /usr/include/EGL /usr/include/KHR "$BUILD/include/"
    python3 - "$BUILD/link/gles.c" <<'PY'
import re,sys,pathlib
header=pathlib.Path('/usr/include/GLES2/gl2.h').read_text()
names=sorted(set(re.findall(r'GL_APIENTRY\s+(gl\w+)\s*\(',header)))
assert len(names)>100
pathlib.Path(sys.argv[1]).write_text('\n'.join('void '+n+'(void) {}' for n in names))
PY
    "$CC" -shared -fPIC -Wl,-soname,libGLESv2.so "$BUILD/link/gles.c" -o "$BUILD/link/libGLESv2.so"
    CPUFLAGS="-mcpu=cortex-a53 -I$BUILD/include"
    GL_LIB="-L$BUILD/link -lGLESv2"
fi
make -C "$BUILD/parallel-n64-$N64" -j"${CORE_JOBS:-4}" platform=unix ARCH="$ARCH" \
    HAVE_GLIDE64=0 HAVE_GLIDEN64=0 HAVE_GLN64=1 HAVE_RICE=1 HAVE_THR_AL=1 HAVE_PARALLEL=0 HAVE_PARALLEL_RSP=0 FORCE_GLES=1 \
    CC="$CC" CXX="$CXX" AR="$AR" CPUFLAGS="$CPUFLAGS" GL_LIB="$GL_LIB" GIT_VERSION=38befe1 \
    LDFLAGS="-shared -Wl,--no-undefined -Wl,--version-script=libretro/link.T -static-libstdc++ -static-libgcc -lpthread -lrt -ldl"
make -C "$BUILD/beetle-ngp-libretro-$NGP" -j"${CORE_JOBS:-4}" platform=unix CC="$CC" CXX="$CXX" AR="$AR" \
    LDFLAGS="-shared -Wl,--no-undefined -Wl,--version-script=link.T -static-libstdc++ -static-libgcc -lm"
cp "$BUILD/parallel-n64-$N64/parallel_n64_libretro.so" "$OUT/"
cp "$BUILD/beetle-ngp-libretro-$NGP/mednafen_ngp_libretro.so" "$OUT/"
printf '%s\n' "$stamp" > "$OUT/.extra-cores-stamp"
