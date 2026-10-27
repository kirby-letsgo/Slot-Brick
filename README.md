<p align="center">
  <img src="assets/slot-brick-banner.png" alt="Slot. for TrimUI Brick and Brick Hammer — retro handheld and cartridge artwork" width="100%">
</p>

<h1 align="center">Slot. for TrimUI Brick — kirby-letsgo fork</h1>

<p align="center">
  A cartridge-first frontend for the <strong>TrimUI Brick</strong> and <strong>Brick Hammer</strong> on stock firmware.<br>
  Pick a system, browse your cartridges, and play.
</p>

<p align="center">
  <a href="https://github.com/kirby-letsgo/Slot-Brick/releases"><strong>Downloads</strong></a> ·
  <a href="#installation">Installation</a> ·
  <a href="docs/BIOS.md">BIOS guide</a> ·
  <a href="brick/README.md">Full user guide</a> ·
  <a href="brick/CHANGELOG.md">Changelog</a> ·
  <a href="https://github.com/kirby-letsgo/Slot-Brick/issues">Report an issue</a>
</p>

This is **kirby-letsgo's fork** of [LordNordyx's TrimUI Brick port](https://github.com/LordNordyx/Slot-Brick), built on [Slot. by Brandon T. Kowalski](https://github.com/BrandonKowalski/slot). It expands the supported systems and adds library, artwork, setup and wireless-transfer features while keeping Slot's cartridge presentation.

This README describes the current `main` branch. The [Dev build](https://github.com/kirby-letsgo/Slot-Brick/releases/tag/dev) includes the newest changes after its build finishes; numbered releases may lag behind. This is an unofficial project under active development. Nintendo 64 and Brick Pro support are experimental, and performance and compatibility vary by game. Report issues to [this fork](https://github.com/kirby-letsgo/Slot-Brick/issues), with your build and logs.

## Features

- **System picker and cartridge shelves:** a six-icon grid using the same wallpapers as the shelves, with system-specific cartridges, HuCards and discs. WonderSwan and WonderSwan Color share a shelf, as do Neo Geo Pocket and Pocket Color.
- **Recently Played and Favorites:** a clock tile shows played games across all systems, newest first; a gold star tile opens starred games together in A–Z order. Both appear only when they have games, preserve each cartridge's system and return to the grid with B.
- **Per-game emulator settings:** set an optional in-game FPS counter and override shader, fast-forward speed, rewind, rumble or turbo from cartridge options or HOME while paused. Game-specific choices inherit system defaults until overridden, and can be reset individually.
- **Live Search across your collection:** see match counts and game previews update on every keystroke, page through matching titles with L1/R1, then press START to browse matching cartridges. Search stays offline and uses an in-memory index; favorites and sorting remain available.
- **Responsive large libraries:** cartridge headers and artwork load on background workers as you browse. Startup no longer waits to cache every new label, and slow artwork reads cannot hold up button handling. See [performance notes and measurements](docs/PERFORMANCE.md).
- **Configurable F1/F2:** separate assignments for the shelf and games. Defaults are F1 for search on the shelf, F1 for saving a state in-game, and F2 for the load-state chooser.
- **Cart Art:** fetch one system or all systems, remember completed attempts, and retry failed or unmatched games. ScreenScraper is an automatic fallback when configured; a failed request is retried once before moving on to the next game.
- **Wireless file transfer:** built-in FTP for games, BIOS files, saves, artwork and wallpapers, accessible from a phone or computer.
- **System Setup:** check emulator files, BIOS requirements and the PICO-8 runtime directly on the Brick, with instructions for missing files.
- **Official PICO-8 and Splore:** use your own Raspberry Pi runtime. PNG carts are their own artwork, and Splore opens from a dedicated cartridge.
- **Native Ports / PortMaster:** a Ports shelf launches installed ports and the PortMaster manager, with Balatro as the first target. A computer-side setup helper installs its supporting runtime and the Balatro wrapper; bring your own `Balatro.exe` or `Balatro.love`. See the [Ports guide](docs/PORTS.md). Physical Brick gameplay validation is still pending.
- **Play and personalize:** save states, per-system screen settings, overlays, cartridge customization where supported, and softcore RetroAchievements on supported games. PICO-8 uses its own menus and saves.
- **Brick integration:** volume and mute, sleep, LED settings, low-battery handling, adaptive performance, and signed updates from this fork. Native PICO-8 uses standby rather than deep sleep.

## Supported systems

Folders below are inside **`Slot/Games/`**. Emulator cores are included; BIOS files and the paid PICO-8 runtime are not. See the [BIOS guide](docs/BIOS.md) for exact paths, filenames and sizes.

| System | Folder | Game formats | Emulator / runtime | BIOS |
| :--- | :--- | :--- | :--- | :--- |
| Game Boy | `GB` | `.gb`, `.gbc` | mGBA | None |
| Game Boy Color | `GBC` | `.gb`, `.gbc` | mGBA | None |
| Game Boy Advance | `GBA` | `.gba` | mGBA / gpSP | Optional |
| NES / Famicom | `NES` | `.nes` | FCEUmm | None |
| SNES / Super Famicom | `SNES` | `.sfc`, `.smc` | Snes9x | None |
| Mega Drive / Genesis | `MD` | `.md`, `.gen`, `.smd`, `.bin` | Genesis Plus GX | None |
| Game Gear | `GG` | `.gg` | Genesis Plus GX | None |
| Master System | `SMS` | `.sms` | Genesis Plus GX | None |
| Nintendo 64 (experimental) | `N64` | `.z64`, `.v64`, `.n64` | ParaLLEl N64 / Rice | None |
| Neo Geo Pocket / Color | `NGP` | `.ngp`, `.ngc` | Beetle NeoPop | None |
| WonderSwan / Color | `WS` | `.ws`, `.wsc` | Beetle Cygne | None |
| Atari Lynx | `LYNX` | `.lnx`, `.lyx`, homebrew `.o` | Beetle Lynx | **Required** |
| Sega 32X | `32X` | `.32x`, `.bin`, `.smd`, `.gen`, `.md` | PicoDrive | None |
| Sega CD / Mega-CD | `SEGACD` | `.cue`, `.chd`, `.iso`, `.m3u` | PicoDrive | **Required, by region** |
| PC Engine / TurboGrafx-16 | `PCE` | `.pce` | Beetle PCE Fast | None |
| PC Engine CD / TurboGrafx-CD | `PCECD` | `.cue`, `.chd`, `.m3u` | Beetle PCE Fast | **Required** |
| Neo Geo (MVS arcade) | `NEOGEO` | FBNeo `.zip`, `.7z` sets | FinalBurn Neo | **Required** |
| PlayStation | `PS1` | `.cue`, `.chd`, `.pbp`, `.m3u`, `.bin`, `.iso`, `.exe` | PCSX ReARMed | Optional, recommended |
| PICO-8 | `PICO8` | `.p8.png`, `.png`, `.p8` | Official Raspberry Pi ARM64 runtime | **Runtime required** |

Cartridge systems accept **one ROM per ZIP/7z** as well as plain ROMs. Neo Geo is different: keep the original FBNeo archives and short filenames intact, and include parent archives for split clones. Disc systems and PICO-8 do not use this automatic unpacking.

For CUE games, keep all referenced BIN/audio tracks with the CUE. M3U playlists group multiple discs; list one relative disc filename per line. Use the [full user guide](brick/README.md) for disc switching and each system's controls.

## Installation

### First install or moving from another Slot fork

1. Back up your entire SD card, including games, BIOS, saves and settings.
2. Download an SD-card ZIP from [this fork's releases](https://github.com/kirby-letsgo/Slot-Brick/releases). For the features on `main`, choose **Dev build**.
3. Prepare an empty FAT32 card. Formatting erases its contents; an existing compatible card does not need reformatting.
4. Extract the ZIP and copy **everything inside `SD-CARD`** to the card's root, including hidden files such as `.slot-boot`.
5. Add your games to the folders above and your own files from the [BIOS guide](docs/BIOS.md). Restore personal `Slot/Games`, `Saves`, `States`, `BIOS`, `Config`, `Labels`, `Overlays` and `Wallpapers` folders from your backup as needed. Keep any `Slot/BootLogo/original*` backups too. Use the new package's `Slot/System`, `Apps`, `Tools` and `trimui` files.
6. Insert the card and power on. If stock firmware appears, open **Apps → Slot** once to set up automatic startup.
7. Open **HOME → System → System Setup** to check your files. Select a system with **A** for details; **X** rechecks.

The card install does not flash the device. An upstream build must be replaced with this fork's card files once: the updater checks this fork's releases and signing key.

### Updating this fork

Use **HOME → Update** over Wi-Fi. Games, saves, BIOS, settings and other personal files are preserved. Dev builds check for newer Dev builds; the update screen also has a channel setting.

For a manual update, back up first, then merge the new `SD-CARD` contents into the card root and replace supplied files. **Do not delete the existing `Slot` folder or reformat for an update.** On macOS, merge folder contents rather than replacing entire folders, which can discard personal files.

## Controls

`HOME` is the small left button below the screen, labelled Menu. `SELECT` is the middle button and `START` the right one.

| Where | Input | Action |
| :--- | :--- | :--- |
| System picker | D-pad, `A` | Choose a system and open its shelf |
| System picker | `L1` / `R1` | Change pages |
| Shelf | Left / Right; Up / Down | Browse games; jump between letters |
| Shelf | `L1` / `R1`; `B` | Switch shelves; return to the system picker |
| Picker / shelf | `X` or default `F1` | Search across systems |
| Shelf | Tap / hold `A` | Resume / start fresh |
| Shelf | `Y`; `SELECT` | Toggle favorite; change sorting |
| Shelf | `START` | Cartridge settings; GBA core picker |
| Menus / shelf | `HOME` | Open settings |
| Emulated game | `HOME` | Pause menu with save, load, reset and quit |
| Emulated game | Hold `HOME` | Save, eject and return to the shelf |
| Emulated game | Default `F1` / `F2` | Save state / open load-state chooser |
| WonderSwan / Lynx | `SELECT` | Rotate picture and controls |
| System | Volume + / −; both together | Volume; toggle mute |
| System | Fn slider; Power | Mute / unmute; sleep / wake |

Configure F1/F2 under **HOME → System → Controls**. PICO-8 has its own controls and saves; use **START → Options → shutdown PICO-8** to return to Slot. See the [full user guide](brick/README.md#controls) for emulator-specific buttons, hotkeys and native PICO-8 behavior.

## Artwork, wallpapers and wireless files

**HOME → Cart Art** lets you choose a system or all systems, then **NEW GAMES**, **RETRY FAILED**, **RETRY NO MATCH** or **ALL MISSING ART**. Attempts are remembered across restarts and existing labels are preserved. Sources are automatic: Slot's label service for GB/GBC/GBA, LaunchBox for other systems, then ScreenScraper when configured. ScreenScraper needs both app developer credentials and a user account; see [ScreenScraper setup](brick/README.md#screenscraper-fallback).

Press **START** on a shelf cartridge and **Y** to search for a label manually, or add PNG labels under `Slot/Labels/<system>/`. [Cart Studio](https://slot-cfw.fyi/) is another option. PICO-8 PNG carts display their own complete artwork and do not use Cart Art.

The system grid and shelves use your backgrounds from `Slot/Wallpapers`. System icons are stored in `Slot/System/Icons`.

For wireless copying, open **HOME → Wi-Fi → File Transfer** and enter the displayed connection details in an FTP client. **START** returns to the library while FTP stays running; a small `FTP` badge by the Wi-Fi icon shows connected clients and transfer activity. Reopen **HOME → Wi-Fi** for connection details and press **B** to stop the server. If games or labels changed, Slot refreshes the library in the background and shows a completion notice. Manual sleep stops FTP; automatic idle sleep is paused while FTP runs. Back up the whole `Slot/Saves` folder, including Neo Geo Pocket's `.flash` files and PICO-8's own saves, as well as `Slot/States`.

## Troubleshooting

- **Missing BIOS or a game will not start:** open **HOME → System → System Setup**, then follow the [BIOS guide](docs/BIOS.md). The checker validates setup files; it cannot guarantee every game's compatibility.
- **Games or systems are missing:** check the folder and extension against the table above, clear the Favorites filter, and allow the background library scan to finish. The grid hides empty shelves. After copying files, stop File Transfer or reopen Slot to refresh.
- **Neo Geo returns to the shelf:** use matching FBNeo game/BIOS sets, preserve archive names, and include a split clone's parent archive. Check the log for missing chips.
- **PICO-8 or Splore fails:** check the ARM64 runtime files, Wi-Fi and clock. Keep the supplied `Slot/System/pico8-bin/wget` helper; details are in `Slot/Logs/pico8.log`.
- **Other crashes:** attach `Slot/Logs/slot.log` and `slot.previous.log` when present to an [issue in this fork](https://github.com/kirby-letsgo/Slot-Brick/issues), with the build, system and game format.
- **Stock menu appears:** launch **Apps → Slot**. **Apps → Check** creates `startup-report.txt` for startup diagnostics. To restore stock startup, run **Apps → Stock**; `Slot/DISABLE_AUTOBOOT` also skips Slot on boot.

The card includes the user guide at `Slot/README.md` and the BIOS guide at `Slot/System/BIOS.md` for offline reference.

## Development

Brick and Brick Hammer code is gated behind the `brick` Cargo feature.

- **Device scripts and apps:** [`brick/`](brick/)
- **Simulator:** [`brick/sim/README.md`](brick/sim/README.md) — the real Brick and Brick Hammer build running on a PC in Docker, with its screen and buttons in a browser. Needs no device, and is how a frontend change is checked before a card is written.
- **SD card:** `sh brick/docker-build.sh` builds the card into `dist-brick/` in Docker, with TrimUI's own toolchain, which only runs on x86-64. On an Apple Silicon Mac, turn on Docker Desktop's Rosetta first (Settings → General → Use Rosetta for x86_64/amd64 emulation on Apple Silicon); under QEMU, rustc crashes.
- **Change history:** [`brick/CHANGELOG.md`](brick/CHANGELOG.md)
- **Dev builds:** every change to `main` is built automatically and published as the [Dev build](https://github.com/kirby-letsgo/Slot-Brick/releases/tag/dev) release, replacing the previous one.

### Making a release

Pushing a `vX.Y` tag builds the card from that tag and publishes it as a release, with
`update.txt` and `update.txt.sig` beside the zip: what **HOME → Update** finds the release by.
`update.txt` is signed with the release key, kept in the `UPDATE_SIGNING_KEY` repository
secret (an Ed25519 private key, PEM); its public half is in `crates/slot/src/update.rs`, and
Slot installs nothing it doesn't verify. Without the secret, a release refuses to build.
Pushing a tag needs write access, so only maintainers can start one. Before tagging:

1. Put the version at the top of [`brick/README.md`](brick/README.md) — it is what the card
   says it is — and in `brick/card/Apps/Slot/config.json`, `brick/card/Slot/System/launch.sh`,
   `brick/card/Slot/System/diagnose.sh` and `PORT_VERSION` in `crates/slot-ui/src/sticker.rs`
   (the About screen).
2. Move the `## Unreleased` notes in [`brick/CHANGELOG.md`](brick/CHANGELOG.md) under a
   `## vX.Y` heading (with `**New since vX.W:**` under it), leaving an empty `## Unreleased`
   above it. Those notes become the release's text.
3. Commit the release changes to `main` and let its checks pass.
4. `git tag vX.Y && git push origin vX.Y`, with the tag on `main`.

### Changelog

Every pull request that changes what is on the card adds a line under `## Unreleased` in
[`brick/CHANGELOG.md`](brick/CHANGELOG.md), written for players: what they can now do or what
was fixed, and where to find it. The Changelog check fails without one. Those lines are the
Dev build's release notes and, at the next tag, the release's. A change players won't notice
(a refactor, a test) takes the `no-changelog` label instead.

The run refuses to publish if the tag is not on `main`, if the card's README names another
version, if the changelog has no section for it, or if that release already exists — and it
runs the simulator's unit tests and scenarios before it builds anything.

## Credits

| Project / contributor | Contribution |
| :--- | :--- |
| [Brandon T. Kowalski](https://github.com/BrandonKowalski/slot) | Original Slot. frontend and Cart Studio |
| [LordNordyx](https://github.com/LordNordyx/Slot-Brick) | TrimUI Brick port on which this fork is based |
| [kirby-letsgo](https://github.com/kirby-letsgo/Slot-Brick) | This fork and its continued development |
| [wargenng](https://github.com/wargenng) | Low-battery LED warning |
| [KrutzOtrem](https://github.com/KrutzOtrem/Trimui-Brick-Overlays) | Bundled Vanilla and CRT overlays |
| Yoshi / Yoshi-kun and [OnionUI Themes](https://github.com/OnionUI/Themes/tree/main/icons/Dot-art%20by%20Yoshi-kun) | System icons, combined with the original TV game icon set |
| Libretro and emulator contributors | mGBA, gpSP, FCEUmm, Snes9x, Genesis Plus GX, PCSX ReARMed, ParaLLEl N64 / Mupen64Plus / Rice, Beetle NeoPop, PicoDrive, Beetle PCE Fast, FinalBurn Neo, Beetle Cygne and Beetle Lynx |
| [Lexaloffle](https://www.lexaloffle.com/pico-8.php) | PICO-8 |
| [minui-pico-8-pak](https://github.com/josegonzalez/minui-pico-8-pak) contributors | PICO-8 sensor compatibility shim and Splore artwork, based on the BBS randomizer cart from oakreef.ie |

Component licenses, source references and notices are in [`licenses/`](licenses/) and ship under `Slot/System/licenses/`. The upstream port credits assistance from Claude Opus; this fork also uses Codex-assisted development.
