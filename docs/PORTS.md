# PortMaster and Balatro

Slot can launch native PortMaster ports on the TrimUI Brick's stock firmware. Open **Ports** in the system grid, then choose a game or **PortMaster** to manage installed ports. Slot releases its display, audio and controller before the native program starts and returns to the shelf when it exits.

This integration is new. Launcher, library, setup and audio-bridge tests pass on the host, and the ARM64 build compiles. **Balatro gameplay on a physical Brick has not yet been verified.** PortMaster's device database supports the Brick, but individual ports still have their own hardware and control requirements.

## First setup on your computer

Update Slot to a build with Ports support, then connect the card to your computer. The card includes `Slot/System/setup-portmaster.py` and `portmaster-runtime.json`. Run the helper with Python 3.9 or newer and an internet connection.

For a Mac with the card mounted as `/Volumes/SLOT`:

```sh
python3 /Volumes/SLOT/Slot/System/setup-portmaster.py /Volumes/SLOT \
  --balatro "$HOME/Downloads/Balatro.exe"
```

Replace the last path with the location of **your purchased `Balatro.exe`**. A purchased Mac `Balatro.love` is also accepted. On Windows, use `py -3` and your card's drive path instead. The helper never downloads the commercial game.

The setup downloads pinned, SHA-256-verified copies of the official TrimUI PortMaster package, Balatro's PortMaster wrapper, and the supporting ARM64 Python/shell libraries from CrossMix. Downloads total roughly 120 MB; leave at least 300 MB free on the card. Python headers, development tools and test suites are omitted. No CrossMix firmware or boot files are installed. Existing PortMaster installations, Balatro launch scripts, purchased game files and saves are kept; use PortMaster itself to update an existing installation.

You can omit `--balatro` and copy the game yourself afterward:

```text
<card>/Data/ports/balatro/Balatro.exe
```

Keep the filename as shown. This EXE supplies the game data to LÖVE; it is not run through Windows emulation. Safely eject the card, boot the Brick, open **Ports → Balatro**. The first launch asks about display layout and buttons, then prepares the handheld build. Start with **Small screen**, **Nunito**, and **Performance On**. This preparation may take a while; subsequent launches reuse it.

## Files and artwork

| Purpose | Card path |
| --- | --- |
| PortMaster manager and its runtimes | `Apps/PortMaster/PortMaster/` |
| Game launchers | `Data/ports/*.sh` |
| Balatro game data, patched build and saves | `Data/ports/balatro/` |
| Slot's additional native runtime | `Slot/System/PortRuntime/` |
| Optional custom cover | `Slot/Labels/PORTS/<launcher name>.png` |
| Native launch log | `Slot/Logs/ports.log` |
| Balatro patch output (latest patch attempt) | `Slot/Logs/balatro-patch.log` |

Slot lists only top-level `.sh` launchers, not helper scripts inside game folders. The small port listing refreshes even when the ROM library index is cached. Games installed in PortMaster appear when Slot returns. Ports also participate in search and Recently Played.

The card-level `Data`, `Imgs` and `Roms` folders are part of PortMaster's TrimUI layout. `Data` contains ports and their saves, `Imgs/PORTS` contains previews, and `Roms/PORTS` is a compatibility folder. Console ROMs remain in `Slot/Games`; do not move them into these folders.

The setup helper copies Balatro's supplied cover if there is no custom one. Other ports can use their standard `Imgs/PORTS/<name>-pre.png` preview or a PNG in `Slot/Labels/PORTS`. Covers fit the case without cropping. The Cart Art scraper excludes native ports.

## Controls, audio and exiting

PortMaster and each game manage their own controls, saves and graphics settings. Slot's emulator save states, rewind, shaders and RetroAchievements do not apply. Balatro's first-run button setup lets its prompts match the buttons printed on the Brick.

Slot's native audio bridge applies its saved volume and mute, including volume buttons and headphone changes, to SDL callback/queued audio and Balatro's OpenAL listener gain. Other audio APIs are not covered by this bridge. Native ports use standby on a short power press; Slot pauses the launched process group and switches off the screen.

Use **the game's own Save/Quit or Exit menu** for a normal return to Slot. Holding HOME is an emergency exit for the whole port process group; it may lose progress not yet saved. Holding power shuts down after stopping the port. Slot's temporary CPU policy is restored afterward.

## Troubleshooting

- **PortMaster support files missing:** run the setup helper on the computer. `HOME → System → System Setup → Ports` checks the installation.
- **Balatro game file missing:** check `Data/ports/balatro/Balatro.exe` (or `Balatro.love`), not `Slot/Games`.
- **Old PortMaster / missing patcher or runtime:** open PortMaster and update it and the Balatro port. The helper preserves an existing installation rather than downgrading it.
- **Balatro patching fails:** send `Slot/Logs/balatro-patch.log` together with `Slot/Logs/ports.log`. The patch screen captures its output, so the native launch log alone may not contain the actual error. Copy the logs before retrying. Keep the purchased EXE and saves in place.
- **Stops at “Fixing Español (México)”:** update Slot and retry. Slot adapts the known August 2026 patcher in a working copy, using its existing runtime translation correction instead of the brittle build-time locale edit. No setup-helper rerun or game-file replacement is needed. Newer/custom patcher versions are left unchanged.
- **Returns immediately, silent audio, or blank screen:** keep `Slot/Logs/ports.log`, PortMaster's `log.txt` and any game log. Report the build and whether other ports work. Physical-device validation is still needed.

On first launch Slot supplies a minimal `System/etc/ex_config` only if none exists, for upstream's TrimUI adapter. It never replaces another firmware's existing configuration. Native integration is intended for the standard `/mnt/SDCARD` mount.

Sources and credits: [PortMaster](https://portmaster.games/), [Balatro port and setup instructions](https://github.com/PortsMaster/PortMaster-New/tree/main/ports/balatro), [CrossMix-OS runtime files](https://github.com/cizia64/CrossMix-OS/tree/206d612fdef83972ed54a17de955125d0c0b3528). Upstream packages retain their own licenses; Python's license is included with its runtime. Balatro belongs to its respective rights holders and must come from your own copy.

Developers can reproduce the locale regression with synthetic game archives (no purchased game required): `python3 brick/test-balatro-patcher.py /path/to/extracted/balatro /path/to/native/7zz`. Use the wrapper pinned in `brick/portmaster-runtime.json`. This exercises the real patcher and launch hook, checks successful archive creation, and confirms incompatible performance patches still fail without changing the input archive or installed script.
