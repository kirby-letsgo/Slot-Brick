# Sourced by non-interactive Bash children through BASH_ENV. Stock TrimUI lacks
# pkill; upstream uses it to release the splash and gptokeyb between LÖVE apps.
slot_portmaster_pkill() {
    local signal=TERM name stat state parent group own_group comm pid found=1
    if [ "${1:-}" = -9 ]; then signal=KILL; shift; fi
    if [ "$#" != 2 ] || [ "$1" != -f ]; then
        echo 'Slot: unsupported pkill arguments' >&2
        return 2
    fi
    name=$2
    case "$name" in sdl2imgshow|gptokeyb|gptokeyb2) ;;
        *) echo "Slot: unsupported pkill target: $name" >&2; return 2 ;;
    esac
    IFS= read -r stat < "/proc/$$/stat" || return 1
    read -r state parent own_group _ <<< "${stat##*) }"
    for stat in /proc/[0-9]*/stat; do
        pid=${stat%/stat}; pid=${pid##*/}
        [ "$pid" != "$$" ] || continue
        IFS= read -r comm 2>/dev/null < "/proc/$pid/comm" || continue
        [ "$comm" = "$name" ] || continue
        IFS= read -r stat 2>/dev/null < "$stat" || continue
        read -r state parent group _ <<< "${stat##*) }"
        # Never signal another port or an unrelated firmware process.
        [ "$group" = "$own_group" ] || continue
        kill -"$signal" "$pid" 2>/dev/null && found=0
    done
    return "$found"
}
if ! command -v pkill >/dev/null 2>&1; then
    pkill() { slot_portmaster_pkill "$@"; }
fi

# The upstream patch UI consumes stdout without logging it. Keep both streams
# while preserving the GUI's pipe and the patch script's actual exit status.
if [ -n "${SLOT_ROOT:-}" ] && [ "$0" = "$(dirname "$SLOT_ROOT")/Data/ports/balatro/tools/patchscript" ]; then
    mkdir -p "$SLOT_ROOT/Logs"
    exec > >(tee "$SLOT_ROOT/Logs/balatro-patch.log") 2>&1
    if [ -f "$SLOT_ROOT/System/balatro-patch-compat.py" ]; then
        slot_patch_copy="$SLOT_ROOT/Config/PortMaster/balatro-patchscript"
        "$SLOT_ROOT/System/PortRuntime/bin/python3" \
            "$SLOT_ROOT/System/balatro-patch-compat.py" "$0" "$slot_patch_copy"
        case $? in
            0) exec "$SLOT_ROOT/System/PortRuntime/bin/bash" "$slot_patch_copy" "$@" ;;
            2) ;; # Unknown upstream version: execute it normally.
            *) echo 'PATCH_FAIL_MSG:Slot could not prepare the patcher working copy.'
               echo 'Patching process failed!'; exit 1 ;;
        esac
    fi
fi
