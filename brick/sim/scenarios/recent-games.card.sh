#!/bin/sh
set -eu
root="$1"
# Use the wallpaper fixture and installed games from the normal grid scenario.
# Keep this history-only golden fixture unstarred; favorites have their own scenario.
sh brick/sim/scenarios/system-grid.card.sh "$root"
printf 'GB\tPocket Test\t120\t30\t0\nMD\tBlast Test\t60\t20\t0\nGBA\tPixel Racer\t600\t10\t0\nGB\tRemoved\t500\t99\t0\n' > "$root/Config/library.txt"
