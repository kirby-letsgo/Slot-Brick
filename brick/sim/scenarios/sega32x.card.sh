#!/bin/sh
set -eu
root="$1"
printf 'simulator stand-in cartridge\n' > "$root/Games/32X/32X Test.32x"
printf 'simulator stand-in cartridge\n' > "$root/Games/32X/Second Test.32x"
