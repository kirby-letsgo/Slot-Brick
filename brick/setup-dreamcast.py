#!/usr/bin/env python3
"""Install an optional, independently distributed Flycast runtime onto a Slot card.

Run on a computer, not on the Brick:
    python3 brick/setup-dreamcast.py --card /path/to/SDCARD
    python3 brick/setup-dreamcast.py --card /path/to/SDCARD --pak DC.pak.zip

Only the Flycast executable, its linked libraries and default controller/config
data are copied. No firmware, ROM, BIOS or emulator save is overwritten.
"""
from __future__ import annotations
import argparse
import hashlib
import os
from pathlib import Path
import shutil
import stat
import sys
import tempfile
from urllib.request import urlopen
from zipfile import ZipFile, ZipInfo

RELEASE = "0.6.5"
URL = (
    "https://github.com/josegonzalez/minui-dreamcast-pak/"
    f"releases/download/{RELEASE}/DC.pak.zip"
)
SHA256 = "3b4d7176f3b3248d8898217c38fa796ff5b36211c0599c48dba5833a24329398"
MAX_BYTES = 100 * 1024 * 1024

FILES = {
    "bin/arm64/flycast": "System/Flycast/flycast",
    "config/emu.cfg": "System/Flycast/emu.cfg",
    "config/mappings/default/SDL_Xbox 360 Controller.cfg":
        "System/Flycast/mappings/SDL_Xbox 360 Controller.cfg",
    "bin/arm64/flycast.LICENSE": "System/licenses/flycast.LICENSE",
    "LICENSE": "System/licenses/minui-dreamcast-pak.LICENSE",
}


def download(dest: Path) -> None:
    print(f"Retrieving pinned Flycast wrapper {RELEASE} from {URL}")
    with urlopen(URL, timeout=45) as source, dest.open("wb") as out:
        total = 0
        while chunk := source.read(1024 * 1024):
            total += len(chunk)
            if total > MAX_BYTES:
                raise ValueError("Flycast release is unexpectedly large")
            out.write(chunk)


def verify(path: Path) -> None:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        while chunk := f.read(1024 * 1024):
            digest.update(chunk)
    if digest.hexdigest() != SHA256:
        raise ValueError("Flycast release SHA-256 mismatch; refusing to install")


def file_for(members: dict[str, ZipInfo], logical: str) -> ZipInfo | None:
    # Accept both root-relative and package-folder-relative archives, but not
    # arbitrary suffix/path matches or ambiguous duplicates.
    matches = [v for k, v in members.items()
               if k == logical or k == f"DC.pak/{logical}"]
    if len(matches) != 1:
        return None
    return matches[0]


def install(source: Path, card: Path) -> list[Path]:
    root = card / "Slot"
    if not root.is_dir():
        raise ValueError("Selected card has no Slot/ directory")
    installed: list[Path] = []
    with ZipFile(source) as archive:
        members = {i.filename: i for i in archive.infolist()}
        selected: dict[str, ZipInfo] = {}
        for src, dst in FILES.items():
            info = file_for(members, src)
            if info is None:
                if src.endswith(".LICENSE") or src == "LICENSE":
                    continue
                raise ValueError(f"Flycast package is missing {src}")
            selected[dst] = info
        for name, info in members.items():
            path = name.removeprefix("DC.pak/")
            if (path.startswith("lib/") and path.count("/") == 1
                    and ".so" in path and not path.endswith("/")):
                selected[f"System/Flycast/lib/{Path(path).name}"] = info
        if not any(k.startswith("System/Flycast/lib/") for k in selected):
            raise ValueError("Flycast runtime libraries are missing")
        if any(info.file_size > MAX_BYTES for info in selected.values()):
            raise ValueError("Unexpectedly large Flycast runtime file")
        for relative, info in selected.items():
            mode = info.external_attr >> 16
            if stat.S_ISLNK(mode):
                raise ValueError(f"Refusing a symlink in Flycast package: {info.filename}")
            target = root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            fd, temp = tempfile.mkstemp(prefix=".flycast-", dir=target.parent)
            try:
                with os.fdopen(fd, "wb") as output, archive.open(info) as incoming:
                    shutil.copyfileobj(incoming, output)
                    output.flush()
                    os.fsync(output.fileno())
                os.chmod(temp, 0o755 if relative == "System/Flycast/flycast" else 0o644)
                os.replace(temp, target)
                installed.append(target)
            finally:
                if os.path.exists(temp):
                    os.unlink(temp)
    return installed


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--card", type=Path, required=True,
                   help="SD card root, containing Slot/")
    p.add_argument("--pak", type=Path,
                   help="locally downloaded DC.pak.zip (offline install)")
    args = p.parse_args()
    try:
        if args.pak:
            verify(args.pak)
            installed = install(args.pak, args.card)
        else:
            with tempfile.TemporaryDirectory() as tmp:
                zip_path = Path(tmp) / "DC.pak.zip"
                download(zip_path)
                verify(zip_path)
                installed = install(zip_path, args.card)
        print(f"Installed {len(installed)} Flycast runtime/config files.")
        print("Next: copy your own dc_boot.bin into Slot/BIOS/DC/")
        print("Put .chd/.gdi/.cdi/.cue game images under Slot/Games/DC/")
        print("Dreamcast remains experimental on stock firmware; see docs/DREAMCAST.md")
        return 0
    except (OSError, ValueError, EOFError) as error:
        print(f"Dreamcast setup: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
