#!/bin/sh
set -eu
root="$1"
card=$(dirname "$root")
sh brick/sim/scenarios/system-grid.card.sh "$root"
mkdir -p "$card/Data/ports/balatro" "$card/Apps/PortMaster/PortMaster/runtimes/love_11.5" "$root/System/PortRuntime/bin" "$root/Labels/PORTS"
printf '#!/bin/bash\n' > "$card/Data/ports/Balatro.sh"
printf '#!/bin/bash\n' > "$card/Data/ports/Pocket Adventure.sh"
cp brick/card/Slot/System/portmaster-launch.sh "$root/System/portmaster-launch.sh"
for f in control.txt pugwash gamecontrollerdb.txt runtimes/love_11.5/love.aarch64; do
    printf 'simulator runtime placeholder\n' > "$card/Apps/PortMaster/PortMaster/$f"
done
for f in bash python3; do printf 'simulator placeholder\n' > "$root/System/PortRuntime/bin/$f"; done
cp "$root/Wallpapers/Grid-test.png" "$root/Labels/PORTS/Balatro.png"
