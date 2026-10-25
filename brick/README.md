# Slot. for TrimUI Brick — v0.44

An unofficial port of **Slot.**, Brandon T. Kowalski's GB/GBC/GBA frontend for the Anbernic RG SP,
to the **original TrimUI Brick** on stock firmware, with NES, SNES, Mega Drive, Game Gear, Master System and PlayStation
added. Port by **LordNordyx**.

> **Note:** Report problems at <https://github.com/LordNordyx/slot-brick>, not to the upstream project.

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

---

## Controls

`HOME` is the left small button below the screen (labelled Menu) and `SELECT` is the middle one.
`A + B` means hold A and press B; `A / B` means either button.

### On the carousel

| Button           | Action                                  |
| ---------------- | --------------------------------------- |
| `Left` / `Right` | Browse games                            |
| `Up` / `Down`    | Previous / next letter                  |
| `L1` / `R1`      | Switch shelves (GBA, GB, GBC, NES, SNES, Mega Drive, PlayStation, Game Gear, Master System) |
| Tap `A`          | Resume from the last save state (a game of several discs: fan them out, pick one, `A` again) |
| Hold `A`         | Start the game fresh                    |
| `START`          | Cart colour, finish, shape and label (GBA: and core) |
| `Y`              | Favorite: a star on the cart, or off again |
| `SELECT`         | Sort: A–Z, last played, most played     |
| `HOME`           | Menu (Screen, Gameplay, Library, Wi-Fi, Cart Art, RetroAchievements, Update, and System: LEDs, Date & Time, Auto Clock, About) |

### In game

| Button                      | Action                                              |
| --------------------------- | --------------------------------------------------- |
| `HOME`                      | Pause: Resume, Save / Load State, Reset, Quit Game  |
| Hold `HOME`                 | Save, eject and return to the carousel              |
| Double-tap `HOME`           | Save state switcher                                 |
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

**Master System:** put `.sms` ROMs in `Slot/Games/SMS` (plain or one per `.zip` / `.7z`).
Games use the bundled Genesis Plus GX core, with no BIOS needed. B / A are buttons 1 / 2;
START presses the console's Pause button, and X / Y provide turbo when enabled. The 192-,
224- and 240-line modes fit a 4:3 screen without clipping. **HOME → Screen → Master System**
has Shader and Overlay. Saves, states, labels and custom overlays use separate `SMS` folders.
RetroAchievements uses the Master System game list. **HOME → Cart Art → GET CART ART**
and label search support Western and Japanese Mark III labels, preserving existing labels.
The shelf uses black cartridges with a narrow top label; tall Japanese artwork is fitted whole.
Custom overlays go in `Slot/Overlays/SMS`; none are bundled.

**Game Gear:** put `.gg` ROMs in `Slot/Games/GG` (one per `.zip` or `.7z` also works).
They run on the bundled Genesis Plus GX core without a BIOS. B / A are buttons 1 / 2,
START is Start, and L1 / R1 choose full screen / the original 160×144 picture.
X / Y provide turbo when enabled. **HOME → Screen → Game Gear** has Shader and Overlay.
Saves and states have their own `GG` folders, and RetroAchievements uses the Game Gear game list.
Carts use a black rounded handheld shell; add PNG labels named after the game in `Slot/Labels/GG`.
**HOME → Cart Art → GET CART ART** downloads Game Gear labels by ROM filename, leaving existing
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
work, `.chd` included. Not yet tested on a Brick: reports welcome ([#41](../../issues/41)).

**Zipped games:** GB, GBC, GBA, NES, SNES, Mega Drive, Game Gear and Master System ROMs can be zipped (`.zip`) or 7z'd (`.7z`), one
ROM in each. The cart takes the archive's name, so `Tetris.gb` zipped as `Tetris.zip` keeps its
saves, states, label and play time. Starting one unpacks its ROM into `Slot/System/Cache/Unpacked`,
where only the last game started is kept. An archive holding more than one ROM, locked with a
password, or packed with anything but deflate (zip) or LZMA / LZMA2 (7z) stays off the shelf, and
`Slot/Logs/slot.log` says why. PlayStation games can't be zipped.

**Cart art:** connect to Wi-Fi, then open **HOME → Cart Art** (greyed out while Wi-Fi is off).
**GET CART ART** gives every game with no label a scan of its real cartridge label; labels you
already have are never replaced. The scans come from the slot art set behind
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

`Slot/Logs/cart-art.txt` lists each game and what it got. Below GET CART ART, every game with a
label is listed: press A twice on one to delete its label, then GET CART ART fetches a new one.

**RetroAchievements:** **HOME → RetroAchievements** (greyed out while Wi-Fi is off) logs in to
your [RetroAchievements](https://retroachievements.org) account and tests the connection.
Your password is sent once, over HTTPS, and never stored: Slot keeps only the login token, in `Slot/Config/retroachievements.txt`; delete that file or choose **LOG OUT** to forget
it. While logged in, games earn achievements in softcore mode (save states, rewind and
fast-forward stay allowed): a small toast slides into the top-right corner with how many a
game has when it starts, and with each unlock and its badge as it happens. A game's badges
are fetched when it starts and kept in `Slot/System/Cache/Badges`. Unlocks made offline are sent once the Brick is back online, while the game is
still running. Hardcore mode isn't available.

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

**Auto Clock:** **HOME → System → Auto Clock** (on to start with) sets the clock from an
internet time server whenever Wi-Fi connects, and every six hours while it stays connected.
The time zone is still yours to choose under **Date & Time**, and summer time is a change you
make there. On the clock screen shown at first start, the date and time fill themselves in
once Wi-Fi connects, unless you have already changed them.

**Brick Pro (experimental):** Slot tells a Brick Pro from a Brick by itself. Its left stick
works as the D-pad, in menus and in games (in a PlayStation game, as the DualShock's left stick), and **HOME → System → LEDs** gains a **Sticks** row
for the lights around the sticks (**Triggers** sets the shoulder lights at the back). The right
stick, the stick clicks and the F1/F2 keys do nothing yet. Not yet tested on a real Brick Pro.

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
