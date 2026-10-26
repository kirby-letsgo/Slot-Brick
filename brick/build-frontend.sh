#!/bin/sh
set -eu
: "${BRICK_TOOLCHAIN:?Set BRICK_TOOLCHAIN to the extracted aarch64-linux-gnu-7.5.0-linaro directory}"
cd "$(dirname "$0")/.."
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-gcc"
export CC_aarch64_unknown_linux_gnu="$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-gcc"
export AR_aarch64_unknown_linux_gnu="$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-ar"
"$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-gcc" -Os -s brick/slot-filecheck.c -o brick/card/Slot/System/slot-filecheck
mkdir -p target
"$BRICK_TOOLCHAIN/bin/aarch64-linux-gnu-gcc" -Os -Wall -Wextra -Werror -shared -fPIC -s \
    brick/pico8-sdl-nosensor.c -o target/pico8-sdl-nosensor.so -ldl
sh brick/build-extra-cores.sh brick
sh brick/build-picodrive.sh brick
sh brick/build-pce.sh brick
sh brick/build-fbneo.sh brick
sh brick/build-handhelds.sh brick
cargo build --locked --profile device --target aarch64-unknown-linux-gnu -p slot --no-default-features --features brick
