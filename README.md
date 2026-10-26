<p align="center">
  <img src="assets/slot-brick-banner.png" alt="Slot. for TrimUI Brick and Brick Hammer — retro handheld and cartridge artwork" width="100%">
</p>

<h1 align="center">Slot. for TrimUI Brick and Brick Hammer</h1>

<p align="center">
  A lightweight frontend for your Game Boy, Game Boy Color, Game Boy Advance, NES, SNES, Mega Drive, Game Gear, and Master System collection.<br>
  Built for the <strong>TrimUI Brick</strong> and <strong>Brick Hammer</strong> running <strong>stock firmware</strong>.
</p>

<p align="center">
  <strong>GB · GBC · GBA · NES · SNES · Mega Drive · Game Gear · Master System</strong> &nbsp;|&nbsp; Stock firmware &nbsp;|&nbsp; Work in progress
</p>

<p align="center">
  <a href="../../releases"><strong>Download</strong></a> ·
  <a href="#installation">Installation</a> ·
  <a href="#controls">Controls</a> ·
  <a href="#troubleshooting">Troubleshooting</a> ·
  <a href="brick/CHANGELOG.md">Changelog</a>
</p>

---

A port of [Slot.](https://github.com/BrandonKowalski/slot), originally developed by **Brandon T. Kowalski** for the Anbernic RG SP. This version adapts Slot. to the TrimUI Brick and Brick Hammer, including its screen, controls, LEDs, and stock firmware startup.

> [!IMPORTANT]
> **Unofficial port · Active development**
>
> This port may contain bugs, incomplete functionality, or missing features. Further improvements are planned, particularly to button shortcuts.
>
> Please report port-specific bugs in **this repository**, rather than in the upstream Slot. project.

## Features

| Feature | Details |
| :--- | :--- |
| **NES and SNES** | Their own shelves of NES and SNES cartridges, running on FCEUmm and Snes9x. No BIOS needed. |
| **Mega Drive** | A shelf of black Mega Drive / Genesis carts, Western or Japanese, running on Genesis Plus GX. No BIOS needed. |
| **Sega 32X** | A 32X shelf running on PicoDrive, with six-button controls, Cart Art and independent saves/settings. No BIOS needed. |
| **Game Gear** | A GG shelf running on Genesis Plus GX, with its own shader and overlay settings. No BIOS needed. |
| **Nintendo 64** | An experimental N64 shelf with its own grey cartridge, Rice graphics and controls for the Brick. |
| **Neo Geo Pocket / Color** | Both handhelds on Beetle NeoPop, with their own wide black cartridges and artwork support. |
| **PICO-8** | Your official Raspberry Pi runtime, PNG cartridges and a built-in Splore launcher. |
| **Master System** | An SMS shelf with its own cartridge shape, settings, saves and Get Cart Art support, running on Genesis Plus GX. No BIOS needed. |
| **Zipped games** | `.zip` and `.7z` ROMs on the GB, GBC, GBA, NES, SNES, Mega Drive, Game Gear and Master System shelves, one ROM in each, under the archive's name. Each unpacks when you start it. |
| **PlayStation (experimental)** | A shelf of discs that spin into a disc slot, on PCSX ReARMed: `.cue`/`.bin`, `.chd`, `.pbp`, `.iso` and `.m3u`, with a disc picker for games of several discs. Not yet tested on a Brick ([#41](../../issues/41)). |
| **Custom cartridges** | Change any cart's colour, finish, and shape from the shelf. |
| **Cart art** | Fetch real cartridge labels for every game at once, or search for one game's label and frame it to fit the cart. |
| **RetroAchievements** | Log in from the menu and earn achievements in softcore mode. |
| **Overlays** | Frames for GB, GBC, GBA, NES and SNES games, by KrutzOtrem, or your own 1024×768 ones (Mega Drive too, but none come with Slot); the game fits itself into the overlay's window. |
| **Brick and Brick Hammer integration** | Screen and control support, plus LED power, brightness, colour, and effects. |
| **Brick Pro (experimental)** | Recognised by itself: the left stick works as the D-pad, and its stick lights have their own LEDs setting. Not yet tested on a Brick Pro; reports welcome ([#45](../../issues/45)). |
| **Sleep support** | Deep sleep with low battery drain. Auto-sleep after 15 minutes idle in a game (Off, 2, 5, 10, 15 or 30 under Gameplay → Auto Sleep), and after 5 minutes at most in the menus. |
| **Save backups** | Each time a game starts, a copy of its battery save goes to `Slot/Saves/Backups`; the last three are kept. |
| **Low battery warning** | F1/F2 stay red at 15% or below and blink rapidly at 10% or below (500 ms cycle), even with decorative LEDs off. Charging restores your lights; at 5%, Slot saves and shuts down (3%, 5%, 10% or off under Gameplay → Low Battery Shutdown). |
| **Adaptive performance** | Adaptive CPU clocks for smooth audio. |
| **Updates** | Update Slot from the menu over Wi-Fi: signed and checked, with your games, saves and settings left alone. |
| **Search** | X on the shelf or HOME → Library → Search Games searches every system offline and shows matching cartridges together. |
| **Library** | The selected game's name on a title card under its cart for a moment, play time beside the battery (minutes, or hours and minutes), favorites with a star, and shelves sorted A–Z, by last played or by most played, or showing only your favorites. |
| **Auto clock** | The clock sets itself from an internet time server whenever Wi-Fi is on. |
| **Wireless file transfer** | HOME → Wi-Fi → File Transfer: use an FTP app on your phone or computer to upload games and artwork or back up saves. B stops transfers and refreshes the library. [Connection details](brick/README.md). |
| **Wireless connectivity** | Wi-Fi, with its status shown beside the clock, and an experimental two-player link over Wi-Fi. |
| **Startup handling** | Support for slow-mounting SD cards. See the known startup issue below. |

## Installation

### 1. Back up your SD card

Copy any important files from your existing SD card to your computer.

> [!WARNING]
> Formatting the SD card erases **all existing data**. Make a backup before continuing.

### 2. Download the latest release

Download `slot-brick-<version>-sd-card.zip` from the [Releases](../../releases) page and extract it on your computer.

### 3. Format the SD card

Use a formatting tool such as [Rufus](https://rufus.ie/) with these settings:

| Setting | Value |
| :--- | :--- |
| File system | `FAT32` |
| Volume label | `SLOT` |

### 4. Copy the installation files

Copy **all contents** of the extracted `SD-CARD` folder to the **root of your SD card**.

> [!IMPORTANT]
> Include hidden files and folders, especially `.slot-boot`. Copy the contents of `SD-CARD`, rather than the folder itself.

### 5. Add your games and saves

Copy your games, save files, save states, and BIOS files from your backup into their respective folders inside `Slot/`. Cartridge ROMs can be zipped (`.zip` or `.7z`, one ROM in each).

> [!TIP]
> Create game labels (cartridge artwork) with [Cart Studio](https://slot-cfw.fyi/) by Brandon T. Kowalski.

### 6. Launch Slot.

Insert the SD card into your TrimUI Brick or Brick Hammer and power on the device. Slot. should launch automatically.

> [!NOTE]
> **Known startup issue:** On first launch or after a restart, the stock firmware menu may appear instead of Slot.
>
> Open **Apps → Slot** to launch it manually. This should restore automatic startup.

## Controls

The three small buttons on the bottom of the Brick and Brick Hammer are **HOME** (left, labelled Menu), **SELECT** (middle), and **START** (right).

**Notation:** `/` means either button. `+` means hold the first button and press the second.

### Carousel

| Input | Action |
| :--- | :--- |
| `Left` / `Right` | Browse the carousel |
| `Up` / `Down` | Jump to the previous / next letter |
| `L1` / `R1` | Switch between the GBA, GB, GBC, NES, SNES, Mega Drive, PlayStation, Game Gear and Master System shelves |
| Tap `A` | Resume the last save state |
| Hold `A` | Start the game fresh |
| `START` | Customise the cartridge (GBA: also choose the emulator core) |
| `Y` | Mark the game as a favorite (a star on its cart), or unmark it |
| `SELECT` | Sort the shelves: A–Z, last played, most played |
| `HOME` | Open the menu: Screen, Gameplay, Library, Wi-Fi, Cart Art, RetroAchievements, Update, and System (LEDs, Date & Time, Auto Clock, About) |

### In game

| Input | Action |
| :--- | :--- |
| `HOME` | Pause: Resume, Save State, Load State (the save state switcher), Reset Game, or Quit Game. Not during multiplayer |
| Hold `HOME` | Save state, eject the cartridge, and return to the carousel |
| Double-tap `HOME` | Open the save state switcher: load, delete, or undo within 30 seconds |
| `SELECT` + `B` or `HOME` + `B` | Toggle LEDs |
| `SELECT` + `HOME` | Open multiplayer Host / Join |
| `SELECT` + `R1` | Save state |
| `SELECT` + `L1` | Load the most recent save state |
| `SELECT` + `A` or `HOME` + `A` | Next shader |
| `SELECT` + `X` | **GBA / SNES / Mega Drive:** switch between original and full screen; saved per game |
| `SELECT` + `Y` | Toggle colour correction |
| `HOME` + `L1` / `R1` | Previous / next overlay for this system |
| `SELECT` + `L2` / `R2` | **GB:** previous / next colour palette |
| `SELECT` + `Up` / `Down` | Adjust brightness |
| `SELECT` + `Left` / `Right` | Adjust the blue light filter |
| `L1` / `R1` | **GB / GBC / NES:** switch between stretched / original picture. On SNES, Mega Drive and PlayStation they are the game's own buttons |
| Hold `L2` | Rewind (not on PlayStation, where `L2` is the game's) |
| Hold `R2` | Fast-forward (**PlayStation:** `HOME` + `R2`, as `R2` is the game's) |
| Double-tap `R2` | Lock fast-forward on; press again to unlock |

### System

| Input | Action |
| :--- | :--- |
| Volume `+` / `−` | Adjust volume |
| `Fn` slider | Mute / unmute |
| Press `Power` | Sleep / wake; game state is saved before sleep |
| Hold `Power` | Shut down or exit to the stock OS |

## Customising cartridges

Press `START` on a game on the shelf to change its cartridge:

- **Colour** (Game Boy, GBA and Famicom): the original, plain colours, or special-edition plastics.
- **Finish** (Game Boy and GBA): solid, see-through, or glitter.
- **Shape:** Game Boy carts can use the original or the Color shape, NES carts the NES or a Famicom cart, SNES carts the US or the Europe / Japan cart, and Mega Drive carts the Western or the Japan cart.
- **Label:** with Wi-Fi on, press `Y` to choose a label. See [Choosing a label](#choosing-a-label).

NES and SNES carts are always their system's grey, and Mega Drive carts always black, like the real ones. Famicom carts came in many colours, so a NES game on the Famicom cart can have any colour.

The cart changes as you go. `A` saves, `B` puts it back, and **Reset to original** restores the game's own look. On a GBA cart, `START` opens the core picker instead: `Left` / `Right` change the core, `Up` the colour, `Down` the finish, and `Y` searches for a label.

### Choosing a label

1. Press `Y` in the cart menu or the GBA core picker. A keyboard opens with the game's name filled in: press `START` to search as it is, or change it first.
2. The labels found appear in a grid. Move with the d-pad, press `A` on the one you want, `Y` to search again, or `B` to go back.
3. Frame the label to fit the cart's cutout. The d-pad moves the picture; hold `L1` and use the d-pad to make the frame bigger or smaller; `R1` evens out its shape; `Y` puts it back as it was. `A` keeps it, `B` goes back to the grid.

## Cart Art and RetroAchievements

Both need Wi-Fi and are greyed out in the `HOME` menu while it is off.

- **Cart Art:** choose one system or **ALL SYSTEMS**, choose a queue, then **FETCH ART FOR THIS QUEUE**. **NEW GAMES** skips completed attempts across restarts; **RETRY FAILED**, **RETRY NO MATCH**, and **ALL MISSING ART** let you retry deliberately. Existing labels stay untouched. Select a game and press `A` to retry it, or `X` twice to delete its label. Sources are automatic: Slot's label service for GB/GBC/GBA and LaunchBox for other systems, followed by ScreenScraper when configured. ScreenScraper errors get one automatic retry, then the game is marked failed and the queue continues. Add your account under **SCREENSCRAPER ACCOUNT**; this also requires app API credentials ([setup](brick/README.md#screenscraper-fallback)).
- **RetroAchievements:** log in to your [RetroAchievements](https://retroachievements.org) account. Only the login token is kept, never your password. Games then earn achievements in softcore mode, with a small toast for each unlock. Hardcore mode isn't available.

## NES and SNES

- Put `.nes` ROMs in `Slot/Games/NES` and `.sfc` / `.smc` ROMs in `Slot/Games/SNES`, zipped or 7z'd if you like.
- No BIOS is needed for either.
- European and Japanese SNES games get the rounder PAL / Super Famicom cart. Slot reads this from the ROM, so ripped ROMs without a region in their name work too.
- Japanese NES games get the small, coloured Famicom cart. NES ROMs don't say where they were sold, so Slot goes by the name, such as `(Japan)`, and by chips only Famicom carts used. For any other game, choose **Famicom** under `START` → **Shape**.
- On SNES, `X`, `Y`, `L1`, and `R1` go to the game.

## Mega Drive

- Put `.md`, `.gen`, `.smd` or `.bin` ROMs in `Slot/Games/MD`, zipped or 7z'd if you like. No BIOS is needed. Games run on Genesis Plus GX.
- Mega Drive carts are always black, in two shapes: Western (US, Europe, Brazil) and Japanese. Japanese games get the rounder Japanese cart: Slot reads where a game was sold from the ROM, then from its name (Japan, Korea, Asia, Taiwan, China or Hong Kong). A game sold in Japan and the West gets the Western cart. Choose the other one under `START` → **Shape**.
- Buttons: `Y` / `B` / `A` are the Mega Drive's A / B / C, `L1` / `X` / `R1` are X / Y / Z, `START` is Start and `SELECT` is Mode. Games made for the 6-button pad get one; older games get the 3-button pad they expect. `L1`, `R1`, `X` and `Y` go to the game.
- The picture is 320×224. Games that use the console's 256-wide mode are widened to the same width, as a TV showed them. **HOME → Screen → Mega Drive** has Shader and Overlay.
- Cart Art fetches Mega Drive labels, and RetroAchievements work.

## Sega 32X

- Put `.32x`, `.bin`, `.smd`, `.gen` or `.md` ROMs in `Slot/Games/32X`, plain or one per `.zip` / `.7z`. PicoDrive is bundled; no BIOS is required.
- The system grid opens a shelf of black cartridges. Saves, states and labels use their own `32X` folders, separate from Mega Drive. Cart Art searches the 32X catalogue and RetroAchievements identifies 32X games.
- Buttons: `Y` / `B` / `A` are A / B / C, `L1` / `X` / `R1` are X / Y / Z, `START` is Start and `SELECT` is Mode. `SELECT` + `X` switches picture size.
- **HOME → Screen → Sega 32X** has separate Shader and Overlay settings. Custom overlays go in `Slot/Overlays/32X`.
- Performance and compatibility still need testing on a physical Brick.

## Game Gear

- Put `.gg` ROMs in `Slot/Games/GG`, either plain or one ROM per `.zip` / `.7z`. No BIOS is needed; Game Gear uses the bundled Genesis Plus GX core.
- `B` / `A` are buttons 1 / 2, and `START` is Start. `L1` / `R1` switch between full screen and the original 160×144 picture. `X` / `Y` provide turbo when enabled.
- **HOME → Screen → Game Gear** has independent Shader and Overlay settings. Put custom 1024×768 PNG overlays in `Slot/Overlays/GG`; none are bundled.
- Saves and states live in `Slot/Saves/GG` and `Slot/States/GG`. RetroAchievements uses the Game Gear game list.
- Carts use a square black Game Gear shell with grip ridges and a SEGA badge. Artwork fits without cropping. Use **HOME → Cart Art → GG → FETCH ART FOR THIS QUEUE** to download Game Gear labels, or add your own at `Slot/Labels/GG/<game name>.png`. Games are matched by filename; No-Intro names work best. Existing labels are kept.

## Master System

- Put `.sms` ROMs in `Slot/Games/SMS`, plain or one per `.zip` / `.7z`. Games use the bundled Genesis Plus GX core; no BIOS is needed.
- `B` / `A` are buttons 1 / 2. `START` presses the console's Pause button; `X` / `Y` provide turbo when enabled.
- The picture fills a 4:3 screen. The usual 192-line mode and extended 224-/240-line modes are fitted without clipping. **HOME → Screen → Master System** has separate Shader and Overlay settings.
- **HOME → Cart Art → SMS → FETCH ART FOR THIS QUEUE** downloads labels by ROM filename, including Western and Japanese Mark III labels. Existing labels are kept. Carts use a black shell with a narrow label across the top; tall Japanese artwork is fitted whole.
- Saves, states, labels and custom overlays use their own `SMS` folders. RetroAchievements uses the Master System game list. No overlays are bundled; custom overlays go in `Slot/Overlays/SMS`.

## Overlays

Slot comes with [KrutzOtrem](https://github.com/KrutzOtrem/Trimui-Brick-Overlays)'s overlays, made for the Brick and Brick Hammer: the *Vanilla* set for GB, GBC and GBA, and the *CRT* set for NES and SNES. Open **HOME → Screen**, choose the system, and pick one on its **Overlay** row. They're off until you do.

To add your own, put 1024×768 PNG overlays in `Slot/Overlays/GB`, `GBC`, `GBA`, `NES`, `SNES`, `MD`, `GG` or `SMS`. The game shows through the part of the overlay that isn't fully opaque. No overlays come with Slot for the Mega Drive, so `MD` starts empty.

The game is scaled to fill that area without stretching. While an overlay is on, the in-game picture size shortcuts do nothing. In game, `HOME` + `L1` / `R1` steps through the system's overlays and Off, naming each as it goes on.

More of KrutzOtrem's sets are in their [repository](https://github.com/KrutzOtrem/Trimui-Brick-Overlays): copy the `GB`, `GBC` and `GBA` folders from a handheld set into `Slot/Overlays`.

## Stock firmware apps

These utilities appear under **Apps** in the stock OS:

| App | Purpose |
| :--- | :--- |
| **Slot** | Launch Slot. and enable automatic startup. |
| **Logo** | Set a custom boot logo or restore the original. |
| **Check** | Report why Slot. did or did not start, without changing anything. |
| **Stock** | Disable Slot. autostart and restore stock startup. |

## Multiplayer

> [!WARNING]
> **Experimental:** Multiplayer has not yet been tested on two physical devices.

1. Install the **same Slot. version** on both devices (Brick or Brick Hammer), using the **same game, emulator core, and BIOS**.
2. In game, press `SELECT` + `HOME` to open the game menu, then choose **Host** or **Join**.
3. Start **Host** on one Brick or Brick Hammer first, then **Join** on the other.

If the connection fails, attach both logs to your issue:

```text
Slot/Logs/multiplayer.log
Slot/Logs/slot.log
```

## Troubleshooting

| Issue or preference | What to do |
| :--- | :--- |
| Stock menu appears instead of Slot. | Launch **Apps → Slot**. For diagnostics, run **Apps → Check** and read `startup-report.txt` on the SD card. |
| Skip Slot. at startup | Create an empty file named `Slot/DISABLE_AUTOBOOT`. |
| Disable auto-sleep | Set **Gameplay → Auto Sleep** to Off. (An empty file named `Slot/DISABLE_AUTO_SLEEP` still turns it off too, and holds the setting at Off.) |
| Restore stock startup | Run **Apps → Stock**. |

### PICO-8 (official runtime)

1. From your [Lexaloffle download](https://www.lexaloffle.com/pico-8.php), extract the **Raspberry Pi** package.
2. Copy **`pico8_64` and `pico8.dat`** together into **`Slot/BIOS/PICO8/`**. The Windows, macOS, x86 Linux and 32-bit `pico8_dyn` builds will not run on this ARM64 launcher. Slot does not include the paid runtime.
3. Put `.p8.png`, `.png` or `.p8` games in **`Slot/Games/PICO8/`**. Reopen Slot or rescan after copying.
4. Select a cartridge and press **A**. The **Splore** cartridge opens PICO-8's own browser; connect Wi-Fi in Slot first to browse online.

A PNG is both the game and its complete cartridge face. Slot displays the whole image and never rewrites it, crops it into another shell, or sends it to Cart Art. Plain `.p8` carts have a placeholder face. If both formats have the same name, the PNG takes precedence. Splore appears when a PICO-8 game or the runtime is installed.

Use the D-pad and A/B to play. **START → Options → shutdown PICO-8** returns to Slot, with the selected cartridge still on the shelf. PICO-8 owns its in-game menus and saves; Slot's save states, rewind, shaders, achievements and emulator hotkeys do not apply. The power button pauses the native process and blanks the screen (standby); tap again to resume. Hold Power and release to shut down. Standby does not suspend the operating system and uses more battery than Slot's normal deep sleep.

PICO-8 writes game saves under **`Slot/Saves/PICO8/`**, settings and Splore's cache under **`Slot/Config/PICO8/`**, and captures under **`Slot/Screenshots/PICO8/`**. These folders and the runtime survive Slot updates. A startup failure returns to the shelf with a message; details are in `Slot/Logs/pico8.log`. First launch seeds the TrimUI controller mapping, which you may customize in `Slot/Config/PICO8/sdl_controllers.txt`.

Slot includes a small PICO-8-only SDL compatibility library so firmware without sensor support can start the runtime. It is installed by the normal Slot update; no extra files are needed from your PICO-8 download.

The launcher and handoff are tested with fixtures; the proprietary runtime, controls, sound, network and standby still need verification on a physical Brick with your copy of PICO-8.

### Nintendo 64 and Neo Geo Pocket

| System | Games folder | ROM formats | Core |
| :--- | :--- | :--- | :--- |
| Nintendo 64 (experimental) | `Slot/Games/N64` | `.z64`, `.v64`, `.n64` | ParaLLEl N64, Rice GLES2 |
| Neo Geo Pocket / Color | `Slot/Games/NGP` | `.ngp`, `.ngc` | Beetle NeoPop |

Both accept one ROM per `.zip` or `.7z` and need no BIOS. They have their own cartridge
shapes, shelves, search results, save states, and **HOME → Screen** Shader / Overlay settings.
Custom overlays go in `Slot/Overlays/N64` or `NGP`; none are bundled. Add PNG labels under
`Slot/Labels/N64` or `NGP`, or use **HOME → Cart Art** to fetch one system's artwork or search
for a label manually. Existing labels are kept. RetroAchievements identifies each system.

N64 renders at 320×240. **Performance and compatibility vary; this has not been verified on a
physical Brick.** The D-pad supplies the analog stick; hold L1 for the digital D-pad instead,
or R2 for the four C buttons. A/B are N64 A/B, X/Y are C-up/down, L2 is Z and L1/R1 are L/R.
START is Start. A Brick Pro's stick supplies the analog stick directly. SELECT + X switches
picture size. Fast-forward is HOME + R2. N64 rewind is disabled because its states are large;
manual saves and loads still work. Game save memory and controller paks are stored together
in `Slot/Saves/N64/<game>.sav`.

On Neo Geo Pocket, B/A are A/B, START is Option, X/Y are turbo, and L1/R1 change picture size.
The core writes in-game saves to `Slot/Saves/NGP/<game>.flash` when you quit or eject normally.
These are separate from Slot's save states in `Slot/States/NGP`, and Slot's `.sav` backup
rotation does not cover `.flash` files. Back them up with FTP or your computer.

## Development

Brick and Brick Hammer code is gated behind the `brick` Cargo feature.

- **Device scripts and apps:** [`brick/`](brick/)
- **Simulator:** [`brick/sim/README.md`](brick/sim/README.md) — the real Brick and Brick Hammer build running on a PC in Docker, with its screen and buttons in a browser. Needs no device, and is how a frontend change is checked before a card is written.
- **SD card:** `sh brick/docker-build.sh` builds the card into `dist-brick/` in Docker, with TrimUI's own toolchain, which only runs on x86-64. On an Apple Silicon Mac, turn on Docker Desktop's Rosetta first (Settings → General → Use Rosetta for x86_64/amd64 emulation on Apple Silicon); under QEMU, rustc crashes.
- **Change history:** [`brick/CHANGELOG.md`](brick/CHANGELOG.md)
- **Dev builds:** every change to `main` is built automatically and published as the [Dev build](../../releases/tag/dev) release, replacing the previous one.

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
3. Merge that as a pull request named `Release vX.Y`; `main` only takes pull requests.
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
| [wargenng](https://github.com/wargenng) | Low-battery LED warning |
| [KrutzOtrem](https://github.com/KrutzOtrem/Trimui-Brick-Overlays) | The bundled overlays: *Vanilla* (GB, GBC, GBA) and *CRT* (NES, SNES), MIT |

- [ParaLLEl N64](https://github.com/libretro/parallel-n64), Mupen64Plus and Rice / glN64 contributors: N64 emulation (GPL-2.0 and component notices in `licenses/`).
- [Beetle NeoPop](https://github.com/libretro/beetle-ngp-libretro), Mednafen and NeoPop contributors: Neo Geo Pocket emulation (GPL-2.0-or-later).

## AI Disclosure
This port was developed with assistance from [Claude Opus](https://claude.ai/). 

---

<p align="center"><sub>Slot. for TrimUI Brick and Brick Hammer · An unofficial port by LordNordyx</sub></p>

PICO-8 is by [Lexaloffle](https://www.lexaloffle.com/pico-8.php). The Splore cartridge artwork comes from [minui-pico-8-pak](https://github.com/josegonzalez/minui-pico-8-pak), based on the BBS randomizer cart from oakreef.ie; its MIT licence is included in `licenses/pico8-splore-MIT.txt`.

PICO-8 sensor compatibility uses the MIT-licensed SDL shim from [minui-pico-8-pak](https://github.com/josegonzalez/minui-pico-8-pak/blob/main/shim/sdl-nosensor.c), adapted in `brick/pico8-sdl-nosensor.c`; its licence ships as `Slot/System/licenses/pico8-sdl-MIT.txt`.
