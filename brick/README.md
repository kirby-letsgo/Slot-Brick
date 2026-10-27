# Slot. for TrimUI Brick — kirby-letsgo fork — v0.44

**kirby-letsgo's fork** of [LordNordyx's TrimUI Brick port](https://github.com/LordNordyx/Slot-Brick),
based on [Slot. by Brandon T. Kowalski](https://github.com/BrandonKowalski/slot).
For the **TrimUI Brick and Brick Hammer** on stock firmware, with a system grid,
cartridge shelves, cross-system search, configurable F1/F2, FTP, artwork queues and
expanded console support. WonderSwan and WonderSwan Color share the `Games/WS` shelf.
Neo Geo Pocket and Color also share a shelf. Nintendo 64 and Brick Pro support are experimental.

Downloads and bug reports: <https://github.com/kirby-letsgo/Slot-Brick>.
This guide follows `main`; the version above is the numbered release baseline.
Newer features arrive in the **Dev build** before the next numbered release.

This is a complete SD card install. You don't need any earlier release, and copying the card
doesn't flash anything to the device.

---

## Installation

1. **Back up.** Power off the Brick and copy your entire current SD card to your PC.
2. **Prepare a card.** Use an empty SD card; a spare is safest. If you reuse your current card,
   verify the backup first, then delete everything on it (no reformat needed).
3. **Copy Slot.** Copy everything **inside** the `SD-CARD` folder to the root of the card,
   including the hidden `.slot-boot` folder. Don't copy the `SD-CARD` folder itself.
4. **Restore your files** from the backup into the matching folders:
   - `Slot/Games`, `Slot/Saves`, `Slot/States`, `Slot/BIOS`
   - `Slot/Config`, `Slot/Labels`, `Slot/Wallpapers`
   - Any `Slot/BootLogo/original*` files

   ⚠️ Do **not** restore old `Slot/System`, `trimui`, `Tools` or `Apps` folders.
5. **Start.** Insert the card and power on. If the stock menu appears instead of Slot, open
   **Apps → Slot** once to set up automatic startup.

**Already running this fork?** Use **HOME → Update** over Wi-Fi. For a manual update,
back up and merge the new package into the card, replacing supplied files while keeping
your games, saves, BIOS and settings. Do not delete `Slot` or reformat for an update.
On macOS, merge folder contents rather than replacing entire folders. Moving from an
upstream build requires copying this fork's files once, because its update signing key differs.

## BIOS and runtime files

See the [BIOS and runtime guide](https://github.com/kirby-letsgo/Slot-Brick/blob/main/docs/BIOS.md)
for exact filenames, sizes, regions and troubleshooting. An offline copy ships at
**`Slot/System/BIOS.md`**. Emulator cores are bundled; supply your own BIOS and PICO-8 files.

| System | Files under `Slot/BIOS/` |
| :--- | :--- |
| Atari Lynx | Required: `lynxboot.img` (512 bytes) |
| Sega CD / Mega-CD | Required for each region played: `bios_CD_U.bin`, `bios_CD_E.bin`, `bios_CD_J.bin` (128 KiB each) |
| PC Engine CD | Required: `syscard3.pce` (256 KiB, optionally with a 512-byte copier header) |
| Neo Geo arcade | Required: complete FBNeo `neogeo.zip`; keep the archive intact |
| PlayStation | Optional, recommended: `scph5500.bin` (Japan), `scph5501.bin` (USA), `scph5502.bin` (Europe), 512 KiB each |
| GBA | Optional: `gba_bios.bin` (16 KiB) |
| PICO-8 | Required: `PICO8/pico8_64` and `PICO8/pico8.dat` from your Raspberry Pi download |

Other supported systems need no external BIOS, including WonderSwan / Color, Neo Geo
Pocket / Color, 32X and PC Engine HuCards. Check files with **HOME → System → System Setup**:
**A** opens details, **X** rechecks. You do not need to create a Splore file; its cartridge
appears when a PICO-8 game or runtime is installed.

---

## Controls

`HOME` is the left small button below the screen (labelled Menu) and `SELECT` is the middle one.
`A + B` means hold A and press B; `A / B` means either button.

### System picker

The library opens with a six-icon grid. Use the D-pad to choose a system,
**L1 / R1** to change pages, and **A** to open its cartridge shelf. **B** on
the shelf returns to the grid; each shelf remembers its selected cartridge.
**X** searches across every system (also **F1** with the default assignments).
**HOME** opens settings. Empty shelves are hidden, including shelves empty under
the Favorites filter. Existing resume-on-boot and single-game startup still apply.

**Recently Played** is the clock tile at the start of the grid. It appears when an
installed game has play history and is selected on startup when opening the grid.
Press **A** to browse played games from every system, newest first. Each cart shows
its system; games never played and games removed from the card are excluded. History
survives restarts and is independent of the Favorites filter and normal shelf sort.
**B** returns to the clock tile. **X** searches the full library; leaving those search
results brings you back to the same recent game. Launch, resume, favorites and cart
settings work as on normal shelves. Ejecting returns to Recently Played with that
game selected. PICO-8 restarts the frontend when it exits and returns to its own shelf.

**Favorites** is the gold star tile beside Recently Played (or before the systems
when no history exists). It appears as soon as an installed game is starred with **Y**,
and disappears when the last star is removed. Press **A** for an A–Z cartridge shelf
of favorites from every system, regardless of the ordinary Favorites-only filter or
shelf sort. Each title keeps its system, save files and controls. **X** searches all
games; **B** returns to the same starred cartridge and a second **B** returns to the
grid. Pressing **Y** here immediately removes a game from the collection. If no
favorites remain, the shelf safely shows NO FAVORITES until you go back. The normal
**HOME → Library → Show: Favorites** filter is separate and is unchanged.

The picker uses the shelf's existing `Slot/Wallpapers` loader and the same
wallpaper texture, crop and dimming. It does not replace or write wallpapers.
System icons live in `Slot/System/Icons`. 32X, Sega CD, PC Engine,
PC Engine CD, Neo Geo, WonderSwan / Color and Lynx are supported; add games to
their folders to show them in the grid.

### On the carousel

| Button           | Action                                  |
| ---------------- | --------------------------------------- |
| `Left` / `Right` | Browse games                            |
| `B`             | Return to the system picker             |
| `Up` / `Down`    | Previous / next letter                  |
| `L1` / `R1`      | Switch between system shelves |
| `X` / default `F1` | Search games across all systems |
| Tap `A`          | Resume from the last save state (a game of several discs: fan them out, pick one, `A` again) |
| Hold `A`         | Start the game fresh                    |
| `START`          | Cart colour, finish, shape and label (GBA: and core) |
| `Y`              | Favorite: a star on the cart, or off again |
| `SELECT`         | Sort: A–Z, last played, most played     |
| `HOME`           | Menu (Screen, Gameplay, Library, Wi-Fi, Cart Art, RetroAchievements, Update, and System: Controls, System Setup, LEDs, Date & Time, Auto Clock, About) |

### In game

| Button                      | Action                                              |
| --------------------------- | --------------------------------------------------- |
| `HOME`                      | Pause: Resume, Save / Load State, Reset, Quit Game  |
| Hold `HOME`                 | Save, eject and return to the carousel              |
| Double-tap `HOME`           | Save state switcher                                 |
| Default `F1` / `F2`         | Save state / open load-state chooser                |
| `HOME + B` or `SELECT + B`  | LEDs on / off                                       |
| `SELECT + HOME`             | Game menu (multiplayer Host / Join)                 |
| `SELECT + R1`               | Save state                                          |
| `SELECT + L1`               | Load the latest save state                          |
| `HOME + A` or `SELECT + A`  | Next shader                                         |
| `SELECT + X`                | GBA/SNES/Mega Drive: original or full screen (saved per game) |
| `SELECT + Y`                | Colour correction on / off (GB, GBC, GBA)           |
| `HOME + L1` / `R1`          | Previous / next overlay                             |
| `SELECT + L2` / `R2`        | GB: previous / next colour palette                  |
| `SELECT + Up` / `Down`      | Brightness                                          |
| `SELECT + Left` / `Right`   | Blue light filter                                   |
| `L1` / `R1`                 | GB/GBC/NES/Game Gear: stretched / original picture (the game's own buttons on SNES, Mega Drive and PlayStation) |
| Hold `L2`                   | Rewind (not on PlayStation, whose `L2` is the game's) |
| Hold `R2`                   | Fast-forward (double-tap to lock it on); PlayStation: `HOME + R2` |

F1/F2 can be reassigned under **HOME → System → Controls**. The table describes
shared emulator controls; see each system's section for exceptions. WonderSwan and
Lynx use **SELECT** to rotate the picture and controls. PICO-8 uses its own menus and saves.

### System

| Button             | Action                                 |
| ------------------ | -------------------------------------- |
| Volume `+` / `−`   | Volume (press both together to mute)   |
| `Fn` slider        | Mute / unmute                          |
| Press `Power`      | Sleep / wake                           |
| Hold `Power`       | Shut down, or exit to the stock OS     |

**Cart colour:** press `START` on a game on the shelf to change its cartridge: any colour and a
solid, see-through or glitter finish for Game Boy and GBA carts, and the shape: original or Color
for Game Boy carts, NES or Famicom for NES carts, US or Europe / Japan for SNES carts, Western or
Japan for Mega Drive carts. NES and SNES carts are always their system's grey and Mega Drive carts
always black, like the real ones; a NES game on the Famicom cart can have any colour, as Famicom
carts came in many. On a GBA cart `START` opens the core picker: `Left` / `Right` swap the core,
`Up` changes the colour and `Down` the finish. The cart changes as you go; `A` saves, `B` puts it
back. Choices are kept in `Slot/Labels/cart_shell.ini`, as `GB/<name>` (and so on per system), so a
game moved between the GB and GBC folders no longer brings its choice with it; a line under the bare
name, from an older Slot or Cart Studio, still counts for that game until you change it in Slot.

**Choosing a label:** with Wi-Fi on, press `Y` in the cart menu (or in the GBA core picker) to
search: the keyboard opens with the game's name filled in, so `START` searches right away, or
change it first (each word only has to begin a word of the name, so `yoshi island` finds
*Super Mario World 2: Yoshi's Island*). It looks through every game of that console in the
places Cart Art uses. The labels found appear in a grid: move with the d-pad, `A` picks one,
`Y` searches again and `B` goes back. Then frame the label to fit the cart's cutout: the d-pad
moves the picture, hold `L1` and use the d-pad to make the frame bigger or smaller, `R1` evens
out its shape and `Y` puts it back as it was. `A` keeps it in place of the label the game had;
`B` goes back to the grid. The labels not picked are deleted when the menu closes. NES games
keep the NES and the Famicom cart's label apart: a label picked goes to the cart shape showing.

**NES:** put `.nes` ROMs in `Slot/Games/NES`; they get their own shelf of NES cartridges and run
on FCEUmm. No BIOS is needed. NES carts are grey; a ROM named as a Japanese release, such as
`Rockman (Japan).nes`, or on a chip only Famicom carts used, gets the small, landscape Famicom
cart instead, in a colour picked from its name (change it under `START` → Colour). NES ROMs don't say where they were
sold, so for other Japanese games choose Famicom under `START` → Shape. Cart Art fetches NES labels too (see below), or put your own picture in
`Slot/Labels/NES`. The picture is the TV's 256 x 224, padded to 4:3 at actual size or
stretched to fill the screen with `L1` / `R1`. RetroAchievements work for NES too.

**SNES:** put `.sfc` or `.smc` ROMs in `Slot/Games/SNES`; they get a shelf of SNES cartridges and
run on Snes9x. No BIOS is needed. European and Japanese games get the rounder PAL / Super Famicom
cart; Slot reads where a game was sold from the ROM, so a ripped `Terranigma.sfc` works too. `L1` / `R1` and `X` / `Y` are the SNES's own buttons (no turbo
on SNES), so `SELECT + X` switches the picture between original and full screen. Cart Art fetches
SNES labels too, and RetroAchievements work.

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

Native PICO-8 respects Slot's saved speaker/headphone volume and mute. Volume keys
and the two-volume-key mute chord also work while playing. Splore downloads use
Slot's bundled HTTPS helper; keep `System/pico8-bin/wget` with the update. Wi-Fi
and a correct clock are needed for new downloads. If downloading fails, check
`Slot/Logs/pico8.log` for the server, connection or certificate error.

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

**Master System:** put `.sms` ROMs in `Slot/Games/SMS` (plain or one per `.zip` / `.7z`).
Games use the bundled Genesis Plus GX core, with no BIOS needed. B / A are buttons 1 / 2;
START presses the console's Pause button, and X / Y provide turbo when enabled. The 192-,
224- and 240-line modes fit a 4:3 screen without clipping. **HOME → Screen → Master System**
has Shader and Overlay. Saves, states, labels and custom overlays use separate `SMS` folders.
RetroAchievements uses the Master System game list. **HOME → Cart Art → SMS → FETCH ART FOR THIS QUEUE**
and label search support Western and Japanese Mark III labels, preserving existing labels.
The shelf uses black cartridges with a narrow top label; tall Japanese artwork is fitted whole.
Custom overlays go in `Slot/Overlays/SMS`; none are bundled.

**Game Gear:** put `.gg` ROMs in `Slot/Games/GG` (one per `.zip` or `.7z` also works).
They run on the bundled Genesis Plus GX core without a BIOS. B / A are buttons 1 / 2,
START is Start, and L1 / R1 choose full screen / the original 160×144 picture.
X / Y provide turbo when enabled. **HOME → Screen → Game Gear** has Shader and Overlay.
Saves and states have their own `GG` folders, and RetroAchievements uses the Game Gear game list.
Carts use a square black Game Gear shell with grip ridges and a SEGA badge. Labels fit without
cropping; add PNG labels named after the game in `Slot/Labels/GG`.
**HOME → Cart Art → GG → FETCH ART FOR THIS QUEUE** downloads Game Gear labels by ROM filename, leaving existing
labels alone. The label search works too. No overlays are bundled; custom 1024×768 PNGs go in `Slot/Overlays/GG`.

**Mega Drive:** put `.md`, `.gen`, `.smd` or `.bin` ROMs in `Slot/Games/MD`; they get a shelf of
black Mega Drive cartridges and run on Genesis Plus GX. No BIOS is needed. Western games (US,
Europe, Brazil) get the Western cart and Japanese ones the rounder Japanese cart: Slot goes by the
regions in the ROM's header, then by the name (`Japan`, `Korea`, `Asia`, `Taiwan`, `China`,
`Hong Kong`), so a game sold in Japan and the West keeps the Western cart. Choose the other cart
under `START` → Shape. `Y` / `B` / `A` are the Mega Drive's A / B / C, `L1` / `X` / `R1` are X / Y
/ Z, `START` is Start and `SELECT` is Mode. Games made for the 6-button pad get one; older games
get the 3-button pad they expect. `L1` / `R1` and `X` / `Y` are the game's own buttons (no turbo
on Mega Drive), so `SELECT + X` switches the picture between original and full screen. The picture
is 320 x 224; games that use the console's 256-wide mode are widened to the same width, as a TV
showed them. **HOME → Screen → Mega Drive** has Shader and Overlay. Cart Art fetches Mega Drive
labels too, and RetroAchievements work.

**PlayStation (experimental):** put games in `Slot/Games/PS1`: `.cue` with its `.bin` tracks,
`.chd`, `.pbp`, `.iso`, or for a game of several discs an `.m3u` listing them. A game may have a
folder of its own; each shows once on the shelf. PlayStation games are discs, not carts: on their
shelf the slot turns into a disc slot, and a disc spins as it goes in. For a game of several discs
`A` fans its discs out; pick one with `Left` / `Right` and `A` puts it in. The fan opens on the
disc you played last. To change discs when a game asks, hold `HOME` to eject, then put the next
disc in: the game carries on with it. They run on PCSX ReARMed, with your BIOS when there is one in
`Slot/BIOS` (`scph5501.bin` and the like), else the core's own. `L2` and `R2` are the game's, so
fast-forward is `HOME + R2`, and there is no rewind: a PlayStation's state is too large to keep
snapshots of on the Brick. Every game gets a DualShock; on a Brick Pro the left stick is its left
stick in game. Games that need more buttons or sticks than the Brick has still start, and play as
far as its buttons allow. Each game's memory card is kept in `Slot/Saves/PS1`. **HOME → Screen →
PlayStation** has Shader, Dithering (on as the PlayStation drew, off for smoother colour) and
Overlay. Cart Art fetches disc scans (below), or put your own round picture in `Slot/Labels/PS1`;
a disc of a game of several can have its own, named after that disc's file. RetroAchievements
work, `.chd` included. Compatibility reports are welcome in
[this fork's issues](https://github.com/kirby-letsgo/Slot-Brick/issues).

**Zipped games:** GB, GBC, GBA, NES, SNES, Mega Drive, Game Gear, Master System, N64,
Neo Geo Pocket / Color, WonderSwan / Color, Lynx, 32X and PC Engine HuCard ROMs can be zipped (`.zip`) or 7z'd (`.7z`), one
ROM in each. The cart takes the archive's name, so `Tetris.gb` zipped as `Tetris.zip` keeps its
saves, states, label and play time. Starting one unpacks its ROM into `Slot/System/Cache/Unpacked`,
where only the last game started is kept. An archive holding more than one ROM, locked with a
password, or packed with anything but deflate (zip) or LZMA / LZMA2 (7z) stays off the shelf, and
`Slot/Logs/slot.log` says why. Disc systems and PICO-8 do not use this unpacking.
Neo Geo arcade instead requires intact FBNeo archives with their original short names.

**Cart art:** connect to Wi-Fi, then open **HOME → Cart Art** (greyed out while Wi-Fi is off).
Choose a system (or **ALL SYSTEMS**), then choose a queue with left/right and select
**FETCH ART FOR THIS QUEUE**. Opening the menu uses the library already in memory; it does not
read all the ROMs. **NEW GAMES** skips completed attempts, including misses and failures, even
after a restart. **RETRY FAILED**, **RETRY NO MATCH**, and **ALL MISSING ART** provide explicit
retries. Each completed result is saved in `Slot/Config/cart-art-history.tsv`; cancelling keeps
that progress. A changed ROM or newly configured ScreenScraper fallback becomes eligible again.
Existing labels are never replaced. This history starts with this version.

GB/GBC/GBA scans come from the slot art set behind
[Cart Studio](https://studio.slot-cfw.fyi) (scans from ScreenScraper), downloaded over HTTPS.
A game is found by the ROM's checksum, or for a trimmed or patched ROM by its file name
(helped by the GBA game code in its header). ROM hacks and homebrew keep the label with their
file name, so they never look like the game they are based on. Each ROM is read once and its
checksum kept in `Slot/System/Cache/crc32.txt`.

NES, SNES, Mega Drive, Game Gear and Master System labels come from the [LaunchBox Games Database](https://gamesdb.launchbox-app.com)'s
pictures of the cartridges: Slot finds the game by its ROM's file name (No-Intro names work best,
for example `Mega Man 2 (USA).nes`) and cuts the label out of a picture of the standard US cart.
Games on the Famicom cart get a Famicom cart's label (from the Japanese release), kept apart in
`Slot/Labels/NES/Famicom` so a NES game can have both: swapping its shape swaps the label. While
a game has only one, both carts show it. SNES games on the rounder Super Famicom / PAL cart get
that cart's label (from the Japanese or European release), and either kind of SNES cart falls
back to the other's label when its own has none. Mega Drive games get the label of their Western or
Japanese cart the same way. Game Gear uses its standard cartridge from any region and saves
labels in `Slot/Labels/GG`. Carts of another shape (Tengen, gold Zelda, EA's and Accolade's Mega
Drive carts, unlicensed) keep their file-name label, as do games LaunchBox has no cart picture of;
the label search still offers those pictures.
PlayStation games get LaunchBox's scan of the disc, kept whole. A game of several discs has a
scan of each, but which is which isn't recorded, so all its discs share one.

`Slot/Logs/cart-art.txt` lists the latest batch. Game rows show only **NO MATCH** and **FAILED**
attempts for the selected system; successful and untried games stay out of this list. Press A
on a game to retry it individually. The queue controls still fetch all eligible games in that
system, including new games.

`Slot/Logs/cart-art-failures.json` keeps unresolved attempts across batches and restarts for
use on a desktop. Each entry has `name` (the ROM filename without its extension), `sha1`, and
`platform` (the game folder code, such as `SMS` or `PS1`). Both failed downloads and unmatched
games are included. Successful retries remove their entries; other systems are preserved.
Hashes are calculated on the background worker even without ScreenScraper credentials, and
ZIP/7z entries use the unpacked ROM's SHA1. A hash is `null` for an unreadable or unsupported
ROM (including CUE/M3U playlists), or an older attempt recorded before hashes were saved.
Starting a fetch fills missing hashes for unchanged older attempts in that system on the
background worker, without scraping them again. The JSON is saved after each completed attempt,
so cancellation keeps completed results. Artwork supplied manually is removed from the failure
export the next time a batch is started for that system.

### ScreenScraper fallback

There is no source selector. Slot tries its usual source first. If it cannot get a label and
ScreenScraper is configured, it tries ScreenScraper using the ROM's SHA1, size, and system.
ZIP/7z games are hashed as the unpacked ROM. Only cartridge labels, cartridge photos that can be
cropped correctly, and disc art are accepted; box art and screenshots are excluded. PNG artwork
is supported. CUE/M3U games keep the existing LaunchBox lookup; CHD matching depends on whether
ScreenScraper knows that exact compressed image. Hashing runs in the background and is cancellable.

In a system's Cart Art menu, open **SCREENSCRAPER ACCOUNT**, enter your ScreenScraper username and
password with the keyboard (START finishes a field), then **SAVE ACCOUNT**. The password is masked.
Saving stores the account; authentication happens on the next fallback request.

ScreenScraper lookup or download errors get one automatic retry after a short pause. If the retry
also fails, that game is marked **FAILED** and the batch continues with the next game. Failed games
remain available in **RETRY FAILED**, including after restarting Slot. A normal no-match response
is recorded as missing without a retry. You can cancel during the retry pause.

ScreenScraper also requires a registered **application developer ID and password**, separate from
player accounts. Dev and release builds read the repository Actions secrets
`SCREENSCRAPPER_DEV_ID` and `SCREENSCRAPPER_DEV_PW` (with that exact spelling) and supply them as
`SLOT_SCREENSCRAPER_DEVID` and `SLOT_SCREENSCRAPER_DEVPASSWORD` when compiling the app. With both
secrets configured, players only need to enter their own ScreenScraper login. Local builds can
set those two `SLOT_` environment variables; alternatively the app owner can add
`developer_id=` and `developer_password=` to `Slot/Config/screenscraper.txt` on their own card.
That file also stores `username=` and `password=`. Do not commit it or include it in shared card
images. The menu says when app API setup is missing, and normal art fetching continues through
the existing source. Enabling fallback does not start a download until you select Fetch.

**RetroAchievements:** **HOME → RetroAchievements** (greyed out while Wi-Fi is off) logs in to
your [RetroAchievements](https://retroachievements.org) account and tests the connection.
Your password is sent once, over HTTPS, and never stored: Slot keeps only the login token, in `Slot/Config/retroachievements.txt`; delete that file or choose **LOG OUT** to forget
it. While logged in, games earn achievements in softcore mode (save states, rewind and
fast-forward stay allowed): a small toast slides into the top-right corner with how many a
game has when it starts, and with each unlock and its badge as it happens. A game's badges
are fetched when it starts and kept in `Slot/System/Cache/Badges`. Unlocks made offline are sent once the Brick is back online, while the game is
still running. Hardcore mode isn't available.

**Function keys:** **F1** opens search on the shelf and saves a state in a game. **F2** opens the save-state chooser in a game; select a state and press A to load it. Set each key separately for the shelf and games under **HOME → System → Controls**, including Off. Other choices include favorite, next/previous system, change sort and menu on the shelf; menu, picture size, mute and next overlay in-game. Choices are kept in `Slot/Config/controls.txt`. Function keys do nothing inside other menus, while entering text, or during multiplayer.

**Wireless file transfer:** connect the Brick and your phone or computer to the same Wi-Fi,
then open **HOME → Wi-Fi → File Transfer** from the shelf. In an FTP app (for example,
FileZilla on a computer or an FTP-capable file manager on your phone), enter the **host**
shown on the Brick, port **2121**, username **slot**, and the displayed temporary password.
Choose **FTP**, **plain / no TLS**, and **passive mode**. The password changes each time
you open File Transfer. FTP is unencrypted, so use your own trusted Wi-Fi.

The connection opens at the **SD-card root**, including `Slot`, `Data`, `Apps`, `Imgs`
and `Roms`. PortMaster games are in `Data/ports`; patch logs are in `Slot/Logs`.
Put console games and cartridge art in their Slot system folders,
for example `Slot/Games/GG/Sonic.gg` and `Slot/Labels/GG/Sonic.png`. You can download backups,
upload files, create folders, rename or delete files. Uploads replace an existing file
only after the transfer finishes; choose **overwrite**, not resume/append, if your FTP
app asks. Keep individual files below 4 GiB (the SD card's FAT32 limit).

Use **START** to return to the game browser while FTP keeps running, including while
browsing shelves or playing emulated games. Launching a native Port or PICO-8 exits Slot
and ends its FTP server. A small `FTP 0` badge to the left of the Wi-Fi icon
shows an active server with no clients; `FTP 1` shows one connected client.
`UP` / `DN` briefly indicates an upload / download in progress. Open
**HOME → Wi-Fi** again to see the FTP password and transfer totals, then press
**B** to **stop the server**. The library rechecks games and artwork in the background
when FTP stops and confirms completion on the shelf; unchanged cartridge art is kept.
Automatic idle sleep is disabled while FTP runs. **Power** still stops transfers before
manual sleep, so finish uploads first. Interrupted uploads discard their temporary
files, leaving previous versions intact. File Transfer can be opened from a shelf,
so eject a game first. Card-level system files, settings and hidden game files are also
accessible. The connection stays within the card; symbolic links are not followed.

**Live Search:** press **X** on the shelf, or open **HOME → Library → Search Games**. Use the D-pad and **A** to type (Y changes the keyboard page, X deletes). The right side instantly shows the number of matches and up to four matching titles with their systems; **L1/R1** pages the preview. Press **START** to browse all matching cartridges in A–Z order, or **B** to cancel without changing the shelf. All search words must match in any order; case does not matter, and searches include games hidden by the Favorites filter. Leave the text empty to browse every game. On the results shelf **Left/Right** browses, **A** plays, **Y** stars, **X** edits the search and **B** returns to your previous shelf. Ejecting returns to the search result. The preview uses in-memory titles and needs no Wi-Fi or ROM reads.

**Library:** when you pick a game, a title card under its cart names it for a moment (**HOME → Library → Title Card** turns it off).
Beside the battery, the shelf shows how long you have played it: in minutes ("200M"), or in hours and minutes ("3H20M", **HOME → Library → Play Time**). `Y` stars a game as a
favorite. `SELECT` sorts the shelves A–Z, by the game played last,
or by the most played, and names the order in the slot; **HOME → Library** has the same order
(**Sort By**) and **Show: Favorites**, which leaves only your favorites on the shelves (all games,
while you have none). Play time, last played and favorites are kept in `Slot/Config/library.txt`.

**Update:** **HOME → Update** (greyed out while Wi-Fi is off) checks
<https://github.com/kirby-letsgo/Slot-Brick/releases> for a newer release of Slot and shows what
is new in it. Choose INSTALL, then press A again: Slot downloads it, checks it,
puts it in place and starts again. It needs 20% battery or a charger. Only Slot's own files
are replaced (`Slot/System`, the `Apps` it came with, `.slot-boot`, the bundled overlays and
this README); your games, saves, states, labels, BIOS files, settings and wallpapers are left
as they are. Every update is signed, and one whose signature or checksum doesn't match is
never installed. If the new version doesn't start, the previous one is put back.

**System Setup:** **HOME → System → System Setup** checks the emulator files,
required BIOS files and PICO-8 runtime for every supported system. Press **A** for
exact filenames, game folders and setup instructions; use **Up/Down** to page through
long details and **X** to recheck after copying files. Sega CD lists each BIOS region
separately. Optional BIOS files are identified without blocking play. Checks read setup
files only, without loading games or contacting a server; “Files ready” does not guarantee
every game is compatible. Supply your own BIOS and licensed PICO-8 runtime files.

**Auto Clock:** **HOME → System → Auto Clock** (on to start with) sets the clock from an
internet time server whenever Wi-Fi connects, and every six hours while it stays connected.
The time zone is still yours to choose under **Date & Time**, and summer time is a change you
make there. On the clock screen shown at first start, the date and time fill themselves in
once Wi-Fi connects, unless you have already changed them.

**Brick Pro (experimental):** Slot tells a Brick Pro from a Brick by itself. Its left stick
works as the D-pad, in menus and in games (in a PlayStation game, as the DualShock's left stick), and **HOME → System → LEDs** gains a **Sticks** row
for the lights around the sticks (**Triggers** sets the shoulder lights at the back). The right
stick and the stick clicks do nothing yet. F1/F2 follow the assignments under **System → Controls**. Not yet tested on a real Brick Pro.

**Wi-Fi status:** the icon beside the clock shows whether Wi-Fi is connected, on but not
reaching the network, or off.

**Overlays:** pick one under **HOME → Screen →** the system **→ Overlay**. Slot comes with
KrutzOtrem's overlays for the Brick (the *Vanilla* set for GB, GBC and GBA, the *CRT* set for NES
and SNES; MIT, see `Slot/System/licenses`). Add your own 1024×768 PNGs to `Slot/Overlays/GB`,
`GBC`, `GBA`, `NES`, `SNES`, `MD`, `GG` or `SMS`. The game is fitted into the part of the overlay that isn't fully
opaque; while one is on, the picture size shortcuts do nothing.
In game, `HOME + L1` / `R1` steps through them and Off. More of KrutzOtrem's
sets are at <https://github.com/KrutzOtrem/Trimui-Brick-Overlays>. No overlays come with Slot for the
Mega Drive: `Slot/Overlays/MD` starts empty.

**Auto-sleep:** the Brick sleeps when it's left alone: in a game after the time set under
**HOME → Gameplay → Auto Sleep** (Off, 2, 5, 10, 15 or 30 minutes; 15 to start with), and on the
shelf and in the menus after 5 minutes, or sooner if Auto Sleep is set shorter. Off turns it off.
An empty file named `Slot/DISABLE_AUTO_SLEEP`, the old way to turn it off, still works: while it
is there, Auto Sleep stays at Off.

**Save backups:** each time you start a game, Slot copies its battery save (a PlayStation game's
memory card too) into `Slot/Saves/Backups/<system>`: `<game> (1).sav` is the newest, then `(2)`
and `(3)`. A save that hasn't changed since the last copy isn't copied again. To go back to one,
copy it into `Slot/Saves/<system>`, rename it `<game>.sav` in place of the save there, then start
the game fresh by holding `A`: resuming would bring back the save the game had in its save state.

---

## Apps

These appear in the stock OS under **Apps**.

| App       | What it does                                                         |
| --------- | -------------------------------------------------------------------- |
| **Slot**  | Starts Slot and sets it to start automatically                       |
| **Logo**  | Sets your own boot logo, or restores the original (see below)        |
| **Check** | Reports why Slot did or did not start, in `startup-report.txt`       |
| **Stock** | Turns off Slot autostart and restores the stock startup              |

### Custom boot logo

1. Put one image (`.png`, `.jpg`, `.jpeg` or `.bmp`) in `Apps/BootLogo/Image`.
2. Open **Apps → Logo**, then restart.

To restore the original, empty the `Image` folder and open **Logo** again.

> **Keep a backup:** the first run saves the original logo to `Slot/BootLogo/original.bmp`.
> Copy that file to your PC; it's the only way back.

---

## Multiplayer (experimental)

1. Install this same version on both Bricks, with the same game, core and BIOS.
2. In game, press `SELECT + HOME`.
3. Choose **Host** on one Brick first, then **Join** on the other.

No router is needed. This hasn't been tested on two physical devices yet. If it fails, send
`Slot/Logs/multiplayer.log` and `Slot/Logs/slot.log`.

---

## Troubleshooting

| Problem                               | Fix                                                                 |
| ------------------------------------- | ------------------------------------------------------------------- |
| Stock menu appears instead of Slot    | Open **Apps → Check** and read `startup-report.txt` on the card     |
| You want to skip Slot at startup      | Create an empty file `Slot/DISABLE_AUTOBOOT`; delete it to re-enable |
| You need stock emulators or RetroArch | They aren't included; keep your original card backup                |

## Emulator credits

N64 uses [ParaLLEl N64](https://github.com/libretro/parallel-n64), based on Mupen64Plus with Rice / glN64 graphics. Neo Geo Pocket uses [Beetle NeoPop](https://github.com/libretro/beetle-ngp-libretro), based on Mednafen and NeoPop. Licences, component notices, pinned sources and build instructions are in `Slot/System/licenses`.

PICO-8 is by [Lexaloffle](https://www.lexaloffle.com/pico-8.php). The Splore cartridge artwork comes from [minui-pico-8-pak](https://github.com/josegonzalez/minui-pico-8-pak), based on the BBS randomizer cart from oakreef.ie; its MIT licence is included in `licenses/pico8-splore-MIT.txt`.

PICO-8 sensor compatibility uses the MIT-licensed SDL shim from [minui-pico-8-pak](https://github.com/josegonzalez/minui-pico-8-pak/blob/main/shim/sdl-nosensor.c), adapted in `brick/pico8-sdl-nosensor.c`; its licence ships as `Slot/System/licenses/pico8-sdl-MIT.txt`.

### Library startup

Slot keeps a disposable game index in `Slot/System/Cache/library-v1.bin`. After the
first scan, it opens the saved library and checks for changes in the background.
Added and removed games appear when the scan finishes and you return to the shelf;
your selected cartridge and library preferences are preserved. An initial scan or
an unreadable index shows **Loading Library**. **Caching New Labels** remains a
separate step for new artwork. No setup is required; deleting the index simply
causes a fresh scan. Startup and background-scan timings are recorded in `Slot/Logs/slot.log`.

## Sega 32X

Put `.32x`, `.bin`, `.smd`, `.gen` or `.md` games in `Slot/Games/32X`, plain
or one ROM per ZIP/7z. PicoDrive is included and no BIOS is required. The 32X
icon appears when games are present. Saves, states, labels and overlays have
separate `32X` folders. Cart Art supports the 32X catalogue.

Y/B/A are Sega A/B/C; L1/X/R1 are X/Y/Z; START is Start and SELECT is Mode.
SELECT + X changes picture size. HOME → Screen → Sega 32X has Shader and Overlay.
Performance and compatibility still need testing on a physical Brick.

## Sega CD / Mega-CD

- Put games in `Slot/Games/SEGACD`. Supported formats are `.chd`, `.cue` with its BIN/audio tracks, and `.iso`. Keep all tracks beside their CUE; loose `.bin` files are hidden. ZIP/7z discs are not supported.
- Supply your own 128 KB BIOS in `Slot/BIOS`, matching each game's region: `bios_CD_U.bin` (USA), `bios_CD_E.bin` (Europe), or `bios_CD_J.bin` (Japan). Filename case matters on case-sensitive filesystems. No BIOS is bundled; missing BIOS files show a setup message.
- Multi-disc games can use an `.m3u` playlist: one relative CUE/CHD/ISO filename per line. Keep the playlist and discs together, optionally in a folder one level below `SEGACD`. The shelf shows one game; opening it lets you choose a disc. When a game asks for the next disc, hold HOME to save and eject, then select that disc from the same playlist; the saved session resumes with it. Keep Eject Save enabled.
- PicoDrive runs Sega CD. Y/B/A are Sega A/B/C; L1/X/R1 are X/Y/Z; START is Start, SELECT is Mode. SELECT + X changes picture size.
- The shelf uses CD faces. Cart Art searches the Sega CD disc catalogue, and custom PNG labels go in `Slot/Labels/SEGACD`. HOME → Screen → Sega CD has Shader and Overlay settings.
- Internal backup RAM and save states are kept per game in `Slot/Saves/SEGACD` and `Slot/States/SEGACD`. Discs in one playlist share that save. The optional RAM cartridge and rewind are disabled. RetroAchievements identifies Sega CD games.
- Host tests cover generated discs, BIOS setup, disc switching, saves and states. Physical Brick performance and commercial-game compatibility still need testing.

## PC Engine / TurboGrafx-16 and PC Engine CD

- Put HuCard `.pce` games in `Slot/Games/PCE`, plain or one per ZIP/7z. They need no BIOS.
- Put CD `.chd`, `.cue` with its BIN/audio tracks, or `.m3u` playlists in `Slot/Games/PCECD`. Keep CUE tracks together; loose BIN/ISO files and ZIP/7z discs are not supported. CHD saves space without unpacking.
- CD games require your **System Card 3 BIOS** at `Slot/BIOS/syscard3.pce` (256 KiB, optionally with a 512-byte copier header). Slot does not include it. A missing/wrong-size file shows setup guidance.
- Beetle PCE Fast is bundled. The system grid opens a HuCard or CD shelf. Cart Art searches the respective catalogue; custom labels go in `Slot/Labels/PCE` or `Slot/Labels/PCECD`.
- **A = I, B = II, SELECT = Select, START = Run**. Controllers start in two-button mode for compatibility. **L2 switches between two and six buttons**; Y = III, X = IV, L1 = V, R1 = VI. Leave two-button mode on for games that do not support six buttons.
- Hold HOME + R2 for fast-forward. HOME + L2 rewinds HuCard games; CD rewind is disabled. SELECT + X changes picture size. HOME → Screen has independent Shader and Overlay pages for each system.
- Saves, states and overlays use separate `PCE` and `PCECD` folders. CD playlists share backup RAM and states across discs. Use relative disc filenames, one per line in the M3U; one nested game folder is supported. To swap, hold HOME to save/eject and choose the next disc from the playlist (keep Eject Save enabled).
- RetroAchievements uses the PC Engine and PC Engine CD console IDs. Native diagnostic tests cover HuCard, CUE, CHD, playlists, backup RAM and states; commercial-game compatibility and performance still need physical Brick testing.

### Neo Geo

- Put FBNeo Neo Geo game archives in **`Slot/Games/NEOGEO/`**. Keep the original short filenames, such as `mslug.zip`; do not extract them. ZIP and 7z are supported. Slot displays full game names and searches both titles and filenames.
- Put the matching **`neogeo.zip`** BIOS archive in **`Slot/BIOS/`**. A copy beside the game archives also works and is hidden from the shelf. BIOS and games are not included.
- Use a current **FBNeo** ROM set. A non-merged set is simplest; split clone sets also need their parent ZIP beside them. MAME/FBA sets may have different chips or checksums. A wrong or incomplete set returns to the shelf; the log lists the missing chips.
- The bundled Neo Geo-only FinalBurn Neo core runs in European **MVS arcade** mode: **Select inserts a coin**, **Start starts**. Brick **B/A/Y/X** are Neo Geo **A/B/C/D** respectively. D-pad moves; Select+X switches picture size, and the configured F1/F2 save/load shortcuts work as usual. Rewind is disabled for this system.
- Saves, memory cards and states stay under `Saves/NEOGEO` and `States/NEOGEO`; screen settings, overlays and labels are independent of Neo Geo Pocket. Manual labels use the ROM stem, for example `Labels/NEOGEO/mslug.png`. Get Cart Art uses LaunchBox cartridge scans, with the existing ScreenScraper fallback when configured.
- RetroAchievements identifies these games under Arcade. Native diagnostic tests cover boot, video, state restore, NVRAM and failed loads. Commercial-game compatibility and performance still need testing on a physical Brick.

### WonderSwan / Color and Atari Lynx

- **WonderSwan / Color share one shelf:** put `.ws` and `.wsc` games in `Slot/Games/WS`.
  Beetle Cygne is included; no BIOS is needed.
- **Atari Lynx:** put `.lnx`, `.lyx` or homebrew `.o` games in `Slot/Games/LYNX`.
  Beetle Lynx is included. Supply your own **512-byte `Slot/BIOS/lynxboot.img`**.
  Missing or wrong-size BIOS files show setup instructions before launch.
- Both systems accept one ROM per ZIP/7z, appear in search and the system picker, and
  have their own cartridge faces, labels, saves, states, shaders and overlays.
  HOME → Cart Art uses the existing LaunchBox lookup and configured ScreenScraper fallback.
  ScreenScraper distinguishes WonderSwan Color by the `.wsc` ROM inside the archive.
- **SELECT rotates the picture and controls**. The image keeps its aspect ratio in
  either orientation. WonderSwan landscape controls: D-pad = X cursor; R2/R1/L2/L1 =
  Y up/right/down/left; A/B = A/B; START = Start. Portrait controls: D-pad = Y cursor,
  face buttons = X cursor, L1/R1 = A/B. HOME + R2 fast-forwards on WonderSwan.
- Lynx: D-pad = directions, B/A = Lynx A/B, L1/R1 = Option 1/2, START = Pause.
- Saves live under `Saves/WS` and `Saves/LYNX`; save states under the matching `States`
  folders. Both use RetroAchievements identification when logged in.
- Core builds and original diagnostic programs were tested on the host. Commercial-game
  compatibility, sound and performance still need checking on a physical Brick.
  Pinned sources and licenses are included under `Slot/System/licenses/handheld-cores.md`.

## Native Ports / Balatro

Open **Ports** from the system grid. Before first use, run the included `Slot/System/setup-portmaster.py` on your computer to install PortMaster and its supporting runtime. It can also copy your purchased `Balatro.exe` using `--balatro`. Full instructions and the current testing limits are in `Slot/System/PORTS.md` on the card, or [the Ports guide](../docs/PORTS.md).
