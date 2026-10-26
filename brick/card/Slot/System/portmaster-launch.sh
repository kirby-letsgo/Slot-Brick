#!/bin/bash
# Slot owns the outer process group; PortMaster owns the installed game scripts.
set -e
SLOT_ROOT=${SLOT_ROOT:-/mnt/SDCARD/Slot}
CARD=$(dirname "$SLOT_ROOT")
PM="$CARD/Apps/PortMaster/PortMaster"
RUNTIME="$SLOT_ROOT/System/PortRuntime"
export PATH="$RUNTIME/bin:/usr/trimui/bin:/usr/bin:/bin:/usr/sbin:/sbin"
export LD_LIBRARY_PATH="$RUNTIME/lib:/usr/trimui/lib:/usr/lib:/lib"
export PYTHONHOME="$RUNTIME"
export PYTHONNOUSERSITE=1
export BASH_ENV="$SLOT_ROOT/System/portmaster-env.sh"
export XDG_DATA_HOME="$CARD/Apps/PortMaster"
export SDL_GAMECONTROLLERCONFIG_FILE="$PM/gamecontrollerdb.txt"
export SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1
export HM_TOOLS_DIR="$CARD/Apps/PortMaster"
export HM_PORTS_DIR="$CARD/Data/ports"
export HM_SCRIPTS_DIR="$CARD/Data/ports"
export PORTMASTER_HOME="$PM"
mkdir -p "$CARD/Data/ports" "$CARD/Data/home" "$CARD/Roms/PORTS" "$CARD/Imgs/PORTS"
# The stock TrimUI adapter has fixed card paths. Never silently launch against a
# different card. Tests use a fake runtime directly, not this device adapter.
if [ "$CARD" != /mnt/SDCARD ]; then
    echo 'PortMaster must be launched on the Brick from /mnt/SDCARD.' >&2
    exit 1
fi
# Upstream ports discover this location before sourcing the TrimUI adapter.
mkdir -p /roms/ports/PortMaster
if [ ! -f /roms/ports/PortMaster/control.txt ]; then
    printf '. "%s/control.txt"\n' "$PM" > /roms/ports/PortMaster/control.txt
fi
if [ ! -f "$CARD/System/etc/ex_config" ]; then
    mkdir -p "$CARD/System/etc"
    # Create only the compatibility environment, never replace an existing CFW file.
    cp "$SLOT_ROOT/System/portmaster-ex-config" "$CARD/System/etc/ex_config"
fi
if [ $# -gt 0 ]; then
    export PM_SCRIPTNAME=$(basename "$1")
    exec "$RUNTIME/bin/bash" "$1"
else
    cd "$PM"
    # The official launcher handles runtime installs, updates and manager restarts.
    exec "$RUNTIME/bin/bash" "$PM/trimui/PortMaster.txt"
fi
