#!/usr/bin/env python3
"""Offline regression tests for the opt-in Dreamcast runtime installer."""
from __future__ import annotations

import importlib.util
from pathlib import Path
import tempfile
import unittest
from zipfile import ZipFile

module_path = Path(__file__).with_name("setup-dreamcast.py")
spec = importlib.util.spec_from_file_location("slot_dreamcast_setup", module_path)
setup = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(setup)

MINIMAL = {
    "bin/arm64/flycast": b"\x7fELF\x02\x01flycast test",
    "config/emu.cfg": b"[config]\nrend.ShowFPS = no\n",
    "config/mappings/default/SDL_Xbox 360 Controller.cfg":
        b"[emulator]\nversion = 3\n",
    "lib/libSDL2.so.0": b"stand-in library",
    "LICENSE": b"fixture licence",
}


class FlycastInstallerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.card = self.root / "SDCARD"
        (self.card / "Slot").mkdir(parents=True)
        self.zip = self.root / "DC.pak.zip"

    def package(self, files=MINIMAL, prefix="DC.pak/") -> None:
        with ZipFile(self.zip, "w") as archive:
            for key, data in files.items():
                archive.writestr(prefix + key, data)

    def test_runtime_extracted_without_touching_saved_content(self) -> None:
        self.package()
        saved = self.card / "Slot/Config/DC/emu.cfg"
        saved.parent.mkdir(parents=True)
        saved.write_text("custom settings\n")
        paths = setup.install(self.zip, self.card)
        self.assertGreaterEqual(len(paths), 4)
        self.assertEqual(
            (self.card / "Slot/System/Flycast/flycast").read_bytes(),
            MINIMAL["bin/arm64/flycast"],
        )
        self.assertEqual(saved.read_text(), "custom settings\n")
        self.assertTrue((self.card / "Slot/System/Flycast/lib/libSDL2.so.0").exists())
        self.assertFalse((self.card / "Slot/BIOS/DC/dc_boot.bin").exists())

    def test_missing_runtime_is_rejected_before_any_file_is_installed(self) -> None:
        self.package({k: v for k, v in MINIMAL.items() if k != "bin/arm64/flycast"})
        with self.assertRaises(ValueError):
            setup.install(self.zip, self.card)
        self.assertFalse((self.card / "Slot/System/Flycast").exists())

    def test_unknown_paths_cannot_escape_the_card(self) -> None:
        self.package({**MINIMAL, "../../outside.txt": b"bad"})
        setup.install(self.zip, self.card)
        self.assertFalse((self.root / "outside.txt").exists())


if __name__ == "__main__":
    unittest.main()
