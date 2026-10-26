#!/usr/bin/env python3
"""Install the official TrimUI PortMaster and Balatro wrapper on an existing Slot card.
Runs on a computer (Python 3.9+); never downloads the commercial game itself.
"""
import argparse
import concurrent.futures
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import tempfile
import urllib.request
import zipfile

HERE = Path(__file__).resolve().parent


def download(item, cache):
    dest = cache / item['sha256']
    if dest.is_file() and hashlib.sha256(dest.read_bytes()).hexdigest() == item['sha256']:
        return dest
    print('Downloading ' + item['name'], flush=True)
    part = dest.with_suffix('.part')
    try:
        with urllib.request.urlopen(item['url'], timeout=90) as response, part.open('wb') as out:
            shutil.copyfileobj(response, out)
        if hashlib.sha256(part.read_bytes()).hexdigest() != item['sha256']:
            raise RuntimeError('Download checksum mismatch: ' + item['name'])
        part.replace(dest)
    finally:
        part.unlink(missing_ok=True)
    return dest


def unpack(archive, dest, accept=lambda _: True):
    with zipfile.ZipFile(archive) as z:
        for entry in z.infolist():
            path = PurePosixPath(entry.filename)
            if path.is_absolute() or '..' in path.parts or '\\' in entry.filename:
                raise RuntimeError('Unsafe archive path: ' + entry.filename)
            if entry.is_dir() or not accept(entry.filename):
                continue
            # FAT cards do not support symlinks. Known pinned archives contain regular files.
            if (entry.external_attr >> 16) & 0o170000 == 0o120000:
                raise RuntimeError('Unexpected archive symlink: ' + entry.filename)
            target = dest / path
            target.parent.mkdir(parents=True, exist_ok=True)
            with z.open(entry) as src, target.open('wb') as out:
                shutil.copyfileobj(src, out)
            target.chmod(0o755 if '/bin/' in '/' + str(path) or entry.filename.endswith('.sh') or (entry.external_attr >> 16) & 0o111 else 0o644)


def python_file(name):
    if name in ('bin/python3', 'bin/python3.11'):
        return True
    return (name.startswith('lib/python3.11/') and not any(
        '/' + part + '/' in name for part in ('test', 'tests', '__pycache__', 'site-packages', 'idlelib', 'tkinter', 'ensurepip')))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('card', type=Path, help='card root, e.g. /Volumes/SLOT')
    parser.add_argument('--balatro', type=Path, help='your purchased Balatro.exe or Balatro.love')
    parser.add_argument('--cache', type=Path, help='reuse verified downloads in this folder')
    args = parser.parse_args()
    card = args.card.resolve()
    if not (card / 'Slot/System/portmaster-launch.sh').is_file():
        parser.error('Update Slot first, then select the card root containing Slot/')
    if args.balatro and (not args.balatro.is_file() or args.balatro.name.lower() not in ('balatro.exe', 'balatro.love')):
        parser.error('--balatro must point to your Balatro.exe or Balatro.love')
    manifest = json.loads((HERE / 'portmaster-runtime.json').read_text())
    with tempfile.TemporaryDirectory(prefix='slot-portmaster-') as temp:
        stage = Path(temp)
        cache = args.cache or stage / 'downloads'
        cache.mkdir(parents=True, exist_ok=True)
        pm_exists = (card / 'Apps/PortMaster/PortMaster/pugwash').is_file()
        game_exists = (card / 'Data/ports/Balatro.sh').is_file()
        items = [x for x in manifest if not (x['name'] == 'PortMaster' and pm_exists)
                 and not (x['name'] == 'Balatro wrapper' and game_exists)]
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            archives = list(pool.map(lambda item: download(item, cache), items))
        runtime = stage / 'runtime'
        runtime.mkdir()
        for item, archive in zip(items, archives):
            if item['name'] == 'Python':
                unpack(archive, runtime, python_file)
            elif item['name'] == 'PortMaster':
                unpack(archive, stage / 'pm', lambda n: n.startswith('Apps/PortMaster/'))
            elif item['name'] == 'Balatro wrapper':
                unpack(archive, stage / 'ports')
            else:
                dest = runtime / item['dest']
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(archive, dest)
                dest.chmod(0o755 if item['dest'].startswith('bin/') else 0o644)
        # Python names bzip2 by SONAME; CrossMix ships the same library as .so.1.
        shutil.copyfile(runtime / 'lib/libbz2.so.1', runtime / 'lib/libbz2.so.1.0')
        shutil.copyfile(runtime / 'bin/curl-aarch64', runtime / 'bin/curl')
        (runtime / 'bin/curl').chmod(0o755)
        shutil.copytree(runtime, card / 'Slot/System/PortRuntime', dirs_exist_ok=True)
        if not pm_exists:
            for file in (stage / 'pm/Apps/PortMaster').rglob('*'):
                if file.is_file(): file.chmod(0o755)
            shutil.copytree(stage / 'pm/Apps/PortMaster', card / 'Apps/PortMaster', dirs_exist_ok=True)
        if not game_exists:
            for file in (stage / 'ports').rglob('*'):
                if file.is_file():
                    target = card / 'Data/ports' / file.relative_to(stage / 'ports')
                    if not target.exists():
                        target.parent.mkdir(parents=True, exist_ok=True)
                        shutil.copy2(file, target)
        for directory in ['Data/ports', 'Data/home', 'Roms/PORTS', 'Imgs/PORTS', 'Slot/Labels/PORTS']:
            (card / directory).mkdir(parents=True, exist_ok=True)
        if args.balatro:
            target = card / 'Data/ports/balatro' / ('Balatro.exe' if args.balatro.suffix.lower() == '.exe' else 'Balatro.love')
            if target.exists():
                print('Keeping existing purchased game file: ' + str(target))
            else:
                shutil.copyfile(args.balatro, target)
        cover = card / 'Data/ports/balatro/cover.png'
        label = card / 'Slot/Labels/PORTS/Balatro.png'
        if cover.is_file() and not label.exists():
            shutil.copyfile(cover, label)
        print('Ready. Safely eject the card, then open Ports → Balatro in Slot.')
        if not args.balatro:
            print('Copy your Balatro.exe into Data/ports/balatro before launching.')
        print('Existing PortMaster, installed port scripts, game data and saves were preserved.')


if __name__ == '__main__':
    main()
