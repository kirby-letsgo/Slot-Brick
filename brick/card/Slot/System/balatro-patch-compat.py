#!/usr/bin/env python3
"""Adapt the known PortMaster patcher without modifying its installed copy.

The August 2026 wrapper already loads patches/loc_fix.lua at runtime. Its
additional build-time es_419 rewrite assumes one exact Lua spelling and can
reject otherwise usable game archives. Use the runtime locale fix. Try optional
source optimizations transactionally: if their version-specific checks fail,
restore the original source files and continue with the handheld build.
"""
import hashlib
from pathlib import Path
import sys

KNOWN_PATCHER = '8df9e30ab15cbc3b3087e5be154211f242b838760bf8c34539bd78183c6d67b1'


def optional_patch(block, files, label):
    """Keep upstream pre/post checks; never retain a partially rewritten group."""
    backup = ''.join(f'    cp "{f}" "{f}.slot-original" || return 1\n' for f in files)
    restore = ''.join(f'      cp "{f}.slot-original" "{f}" || return 1\n' for f in files)
    remove = ''.join(f'    rm -f "{f}.slot-original" || return 1\n' for f in files)
    return (backup + '    slot_optional_patch() {\n' + block + '\n    }\n'
            '    if ! slot_optional_patch; then\n' + restore +
            f'      echo "Slot: skipped optional {label} optimizations; original source restored for this game version."\n'
            '    fi\n' + remove)


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

    groups = [
        ('    expect_occurrences 1 game.lua', '\n  fi\n',
         ['game.lua', 'cardarea.lua', 'engine/text.lua'], 'card animation/text'),
        ("    sed -i '/^vec4 effect", '\n    "$SEVENZIP" u -aoa "$WORKFILE" resources/shaders/CRT.fs',
         ['resources/shaders/CRT.fs', 'resources/shaders/background.fs'], 'shader'),
    ]
    for begin, finish, files, label in groups:
        start = text.index(begin)
        end = text.index(finish, start)
        text = text[:start] + optional_patch(text[start:end], files, label) + text[end:]
        # Clean backup files after a failed/cancelled attempt as well.
        cleanup = ''.join(f'  rm -f "$PATCHDIR/{f}.slot-original"\n' for f in files)
        text = text.replace('cleanup_patch_work() {\n', 'cleanup_patch_work() {\n' + cleanup, 1)
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
    print('Slot: applying Balatro locale and optional-performance compatibility fixes to a working copy.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
