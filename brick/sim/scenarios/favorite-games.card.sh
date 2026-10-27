#!/bin/sh
set -eu
root="$1"
sh brick/sim/scenarios/system-grid.card.sh "$root"
# Both are installed; a missing favorite would not be shown in the collection.
printf 'GB\tPocket Test\t0\t0\t1\nMD\tBlast Test\t0\t0\t1\nGB\tMissing\t0\t0\t1\n' > "$root/Config/library.txt"
