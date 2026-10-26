#!/usr/bin/env python3
"""Adapt the known PortMaster patcher without modifying its installed copy.

The August 2026 wrapper already loads patches/loc_fix.lua at runtime. Its
additional build-time es_419 rewrite assumes one exact Lua spelling and can
reject otherwise usable game archives. Keep the runtime fix and all game,
performance and archive checks; omit only that redundant locale edit.
"""
import hashlib
from pathlib import Path
import sys

KNOWN_PATCHER = '8df9e30ab15cbc3b3087e5be154211f242b838760bf8c34539bd78183c6d67b1'


def adapt(source):
    if hashlib.sha256(source).hexdigest() != KNOWN_PATCHER:
        return None  # A newer/custom patcher owns its own compatibility fixes.
    text = source.decode('utf-8')
    start = text.index('  # Stock es_419 copies Russian')
    end = text.index('  if [ "$PERF_OPTIMIZATIONS" -eq 1 ]; then', start)
    text = text[:start] + '  echo "Using the runtime Spanish translation fix (loc_fix.lua)."\n\n' + text[end:]
    text = text.replace('$SOURCES $MODULES localization/es_419.lua >/dev/null',
                        '$SOURCES $MODULES >/dev/null')
    start = text.index('  "$SEVENZIP" x -so "$WORKFILE" localization/es_419.lua')
    end = text.index('\n\n', start)
    text = text[:start] + text[end:]
    return text


def main():
    source, destination = map(Path, sys.argv[1:])
    patched = adapt(source.read_bytes())
    if patched is None:
        print('Slot: leaving this newer/custom Balatro patcher unchanged.')
        return 2
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_suffix('.tmp')
    temporary.write_text(patched, encoding='utf-8')
    temporary.replace(destination)
    print('Slot: applying Balatro locale compatibility fix to a working copy.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
