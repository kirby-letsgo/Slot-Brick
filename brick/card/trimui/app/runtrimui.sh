#!/bin/sh
# SLOT_BRICK_BOOT_V2
# Startup hook v5. Waits (up to 30 s) until the SD card is really mounted.
# Earlier hooks only waited for a "Slot" folder, but a stray Slot folder can
# exist on internal storage under /mnt/SDCARD before the card is mounted; the
# hook then saw no AUTOBOOT and went to the stock menu.
# Decisions are logged to /tmp/slot-hook.log and Slot/Logs/hook.log.
# Steps are timed in /tmp/slot-boot-trace; Slot keeps them in Slot/Logs/boot-times.log.
CARD=/mnt/SDCARD
PREVIOUS=/usr/trimui/bin/runtrimui-slot-backup.sh
MOUNTS=${SLOT_HOOK_MOUNTS:-/proc/mounts}
LOG=/tmp/slot-hook.log
TRACE=${SLOT_BOOT_TRACE:-/tmp/slot-boot-trace}
stamp() { read up _ < "${SLOT_UPTIME:-/proc/uptime}" && echo "$up $1" >> "$TRACE"; } 2>/dev/null
{ printf '' > "$TRACE"; } 2>/dev/null
stamp 'hook: started'
early=no
[ ! -d "$CARD/Slot" ] || early=yes
count=0
until grep -q " $CARD " "$MOUNTS" 2>/dev/null || [ "$count" -ge 300 ]; do
    sleep 0.1
    count=$((count + 1))
done
mounted=no
grep -q " $CARD " "$MOUNTS" 2>/dev/null && mounted=yes
[ "$mounted" = no ] || stamp 'hook: card mounted'
has() { if [ -e "$1" ]; then echo yes; else echo no; fi; }
note="slot hook v5: uptime $(cut -d' ' -f1 /proc/uptime 2>/dev/null)s, waited $count x 0.1s, card mounted=$mounted, Slot folder before mount=$early, AUTOBOOT=$(has "$CARD/Slot/AUTOBOOT") DISABLE=$(has "$CARD/Slot/DISABLE_AUTOBOOT") boot.sh=$(has "$CARD/.slot-boot/boot.sh")"
echo "$note" >> "$LOG" 2>/dev/null
[ "$mounted" = no ] || echo "$note" >> "$CARD/Slot/Logs/hook.log" 2>/dev/null
if [ "$mounted" = yes ] && [ -f "$CARD/Slot/AUTOBOOT" ] && [ ! -f "$CARD/Slot/DISABLE_AUTOBOOT" ] && [ -f "$CARD/.slot-boot/boot.sh" ]; then
    sh "$CARD/.slot-boot/boot.sh"
fi
rm -f "$TRACE"
if [ -f "$PREVIOUS" ] && ! grep -q '^# SLOT_BRICK_BOOT_V[12]$' "$PREVIOUS"; then
    exec /bin/sh "$PREVIOUS"
fi
exit 1
