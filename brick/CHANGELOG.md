# Changelog — Slot. for TrimUI Brick

## Unreleased

- Native ports: restore HOME-hold exit and live volume/mute controls by reading the Brick's SDL joystick during native gameplay. The power-only evdev reader could not see these buttons. PICO-8 also regains live volume controls; its existing exit behavior is unchanged.

- Balatro: optional source/shader optimizations now restore their original files and continue when a game's code does not match the patcher's expected version (including the reported `cardarea.lua` mismatch). Required files and archive validation remain mandatory; skipped optimizations are recorded in the patch log.

- Ports now use compact PC game boxes with burgundy title spines, textured covers and raised edges. Existing artwork stays uncropped; missing covers get a title design and PortMaster gets a toolbox cover. Wallpaper and shelf navigation keep their existing behavior.
- FTP File Transfer now opens at the SD-card root, including PortMaster's `Data/ports`, `Apps` and all of `Slot` (logs, settings and system files). Supports root-level file management and hidden game files while keeping traversal/symlink restrictions and atomic uploads.

- Balatro: the known PortMaster wrapper no longer rejects game archives during its redundant Spanish translation edit. Use its existing runtime translation fix while retaining performance and archive validation. Apply the compatibility change to a working copy; installed port scripts, purchased game files and saves stay intact.

- PortMaster on stock TrimUI: provide missing splash/controller-helper cleanup within the launched port's process group. Capture Balatro's patch output separately in `Slot/Logs/balatro-patch.log` so patch failures no longer disappear inside the patch UI. Document PortMaster's card-level folders.

- Native Ports shelf and PortMaster launch/return, including search, recent games, uncropped covers and setup guidance. Added a checksum-verified computer setup helper for the official TrimUI PortMaster, native runtime dependencies and Balatro wrapper; copy your own Balatro.exe or Balatro.love. SDL/OpenAL volume bridge, process-group standby/emergency exit and separate port logs. See `docs/PORTS.md`; physical Brick gameplay validation remains pending.

- Faster library startup and search: constructing or rearranging shelves no longer opens every ROM/archive to identify its cartridge shape. Artwork workers resolve shapes as needed, and search keeps already prepared shapes and textures. New labels are cached as you browse instead of blocking startup behind Caching New Labels. Visible artwork waits share a 2 ms budget per frame, allowing navigation while slow artwork loads. Added regression tests and a reproducible large-library benchmark; see `docs/PERFORMANCE.md` for host measurements and hardware-testing limits.

- Recently Played: a clock tile at the start of the system grid opens played games across all systems, newest first, with each cart's system shown. Uses existing play history across restarts, skips removed/unplayed games, and includes non-favorites. B returns to the grid; search can open from the collection and return to the same game.

- Updated the README and card guide for the kirby-letsgo fork and its supported systems. Added a BIOS/runtime guide with exact filenames, sizes, region requirements and System Setup instructions, also included on the card at `Slot/System/BIOS.md`.

- WonderSwan / Color on one shelf (`Games/WS`) and Atari Lynx (`Games/LYNX`): bundled Beetle cores, cartridge faces, Yoshi icons, art lookup, search, saves/states, settings, RetroAchievements identification and portrait rotation with SELECT. Lynx requires your 512-byte `BIOS/lynxboot.img`; System Setup checks it.

- Native PICO-8 now uses Slot’s startup volume/mute, live volume keys and mute chord, and separate speaker/headphone levels. Splore uses a packaged HTTPS download helper with built-in CA roots and one retry, fixing reliance on missing or outdated firmware wget. Download errors appear in `Logs/pico8.log`.

- PC Engine / TurboGrafx-16 and PC Engine CD: HuCards in `Games/PCE`, CUE/CHD/M3U discs in `Games/PCECD`, bundled Beetle PCE Fast, HuCard/disc faces, artwork, search, independent saves/settings and RetroAchievements identification. CD needs your `BIOS/syscard3.pce`. Two-button controls by default; L2 switches to six buttons. Includes libretro playlist disc control and missing-BIOS guidance.

- Sega CD / Mega-CD: games in `Slot/Games/SEGACD` run on PicoDrive, including CHD, CUE with tracks, ISO and multi-disc M3U playlists. Add your own region BIOS in `Slot/BIOS` (`bios_CD_U.bin`, `bios_CD_E.bin`, `bios_CD_J.bin`). Includes the system icon, disc shelf and picker, disc art, six-button controls, separate saves/settings and RetroAchievements identification. Missing BIOS files show setup guidance. Physical Brick testing is still needed.

- HOME → Screen now scrolls through six full-size system rows, keeping the selected system visible and the button hints fixed below the list.

- Sega 32X: put ROMs in `Slot/Games/32X`, plain or one per ZIP/7z. Bundled PicoDrive runs them without a BIOS, using the 32X system icon, black cartridge shelf, six-button controls, separate saves/states/settings, Cart Art and RetroAchievements identification. HOME → Screen → Sega 32X has Shader and Overlay. Tested with a homebrew ROM on the host; physical Brick performance and compatibility still need testing.

- System picker: a clean six-icon grid before the shelves, using the same wallpaper. A opens a system, B returns, and L1/R1 change pages. Empty systems are hidden; global search and each shelf's selection are preserved. Includes Yoshi-kun icons for all supported systems, plus 32X, Sega CD, PC Engine, PC Engine CD and Neo Geo ready for future support.

- Faster startup with large libraries: remember the game index between boots, show the shelf from it, and scan for changes in the background. The first scan shows Loading Library instead of a black screen; background results are applied when back on the shelf.

- Fixed N64 crashing during graphics startup on the Brick, and official PICO-8 exiting because the firmware's SDL library lacks sensor support. Both fixes are included in the normal update; keep your existing games and PICO-8 runtime files.

- Cart Art retries a failed ScreenScraper lookup or download once. If it fails again, that game is remembered as failed and scraping continues with the next game.

- Official PICO-8: add `pico8_64` and `pico8.dat` from your Raspberry Pi download to `Slot/BIOS/PICO8`, and carts to `Slot/Games/PICO8`. PNG carts show their own complete cartridge images; Splore has a dedicated cartridge. Games run in PICO-8 and return to the shelf when you quit.

- Cart Art lists failed and unmatched attempts with individual retries, and exports their names, SHA1 hashes and systems to `Slot/Logs/cart-art-failures.json`.

- Nintendo 64 (experimental): put `.z64`, `.v64` or `.n64` games in `Slot/Games/N64`, plain or one per ZIP/7z. Games use ParaLLEl N64 with Rice graphics at 320×240. The D-pad moves the analog stick; hold R2 for C buttons or L1 for the digital D-pad. A/B are A/B, X/Y are C-up/down, L2 is Z, and L1/R1 are L/R. HOME + R2 fast-forwards; N64 rewind is disabled. Performance and compatibility vary; physical Brick testing is still needed.
- Neo Geo Pocket and Pocket Color: put `.ngp` and `.ngc` games in `Slot/Games/NGP`, plain or one per ZIP/7z. Beetle NeoPop runs both without a BIOS. B/A are A/B and START is Option. In-game flash saves use `Slot/Saves/NGP/*.flash`; quit the game normally to flush them.
- Both new systems have their own cartridge shells, search results, save-state folders, RetroAchievements identification and HOME → Screen settings. Cart Art and label search support both; downloaded artwork fits the labels without cropping its sides.

- HOME → Cart Art: choose one system or all systems, remember completed attempts across restarts, and retry failed or unmatched games separately. Add a ScreenScraper account for automatic SHA1 fallback when app API credentials are configured; existing labels are kept.
- Fixed Cart Art freezing on large libraries: opening the screen and finishing downloads no longer rescan every game. PlayStation scans no longer read entire disc images as text, reducing startup delays and memory use with large CHD collections.
- Game Gear now has its own square black cartridge with grip ridges and a SEGA badge, replacing the Game Boy Color shell. Existing labels refresh automatically and show their full artwork.
- Wireless file transfer: HOME → Wi-Fi → File Transfer shows FTP connection details for your phone or computer. Upload or download games, saves, states, artwork, BIOS files, overlays and wallpapers. Press B to stop and refresh the shelves; the Brick stays awake while transfers are open.
- HOME → System → Controls assigns F1 and F2 separately on the shelf and in games. Defaults: F1 searches on the shelf and saves a state in-game; F2 opens the save-state chooser in-game. Settings survive reboot. Brick Pro uses its own F1/F2 keys, leaving stick clicks unassigned.
- Search all your games with X on the shelf or HOME → Library → Search Games. Type part of a name and press START to browse matching cartridges from every system together, with each system named. Searches include non-favorites; X edits the search, B returns to your shelf, and an empty search shows all games. Works offline.
- Fixed Master System labels being cropped at the sides: the complete artwork now fits inside the cartridge label, including in the label picker. Cached labels refresh after updating; no download is needed.
- Sega Master System: put `.sms` ROMs in `Slot/Games/SMS` (plain, zipped or 7z'd) for an SMS shelf with its own black cartridges, saves, states and RetroAchievements support. Games run on the bundled Genesis Plus GX core without a BIOS. B / A are buttons 1 / 2 and START is the console's Pause button. HOME → Screen → Master System has Shader and Overlay, and GET CART ART and label search fetch Western and Japanese Mark III labels without replacing existing labels. Custom overlays go in `Slot/Overlays/SMS`; none are bundled.
- HOME → Cart Art → GET CART ART now fetches Game Gear cartridge labels, matched by the ROM filename. The label search supports Game Gear too. Labels go in `Slot/Labels/GG`, and existing labels are left alone.
- Sega Game Gear: put `.gg` ROMs in `Slot/Games/GG` (plain, zipped or 7z'd) for a GG shelf using the bundled Genesis Plus GX core, with no BIOS needed. B / A are buttons 1 / 2 and START is Start; L1 / R1 switch picture size. HOME → Screen → Game Gear has its own Shader and Overlay settings. Saves, states, labels and overlays use separate GG folders, and RetroAchievements identifies Game Gear games. Carts use a black rounded handheld shell; local labels work.
- Removed boot and wake timing logs to avoid extra logging during startup.
- Faster startup: the shelf appears sooner, with lighter boot scripts and icons, and sound and games prepared while the screen opens.
- HOME → Gameplay → Auto Sleep: how long a game can be left alone before the Brick sleeps: Off, 2, 5, 10, 15 or 30 minutes (15, as before, to start with). On the shelf and in the menus it still sleeps after 5 minutes, or sooner if you choose less. Off replaces the `Slot/DISABLE_AUTO_SLEEP` file, which still works.
- HOME → Update on a Dev build now looks for the newest Dev build straight away. Before, it looked for releases until you unlocked the Channel row and chose Dev. Choosing a channel there is still kept.
- Save backups: each time you start a game, its battery save is copied into `Slot/Saves/Backups`, and the last three copies are kept. To go back to one, put it in place of the game's save in `Slot/Saves` (named as the game, `.sav`), then hold A on the game to start it fresh.
- HOME → Update now gets Slot from kirby-letsgo/Slot-Brick, signed with that build's own key. A Brick on a LordNordyx build has to have this one copied onto its card once; after that, HOME → Update keeps it up to date.
- Sega Mega Drive: put `.md`, `.gen`, `.smd` or `.bin` ROMs in `Slot/Games/MD` (zipped or 7z'd too) and they get a shelf of black Mega Drive carts of their own, on Genesis Plus GX. No BIOS is needed. Japanese games get the Japanese cart, and a game sold in the West too keeps the Western one; change it under START → Shape.
- Mega Drive games: Y / B / A are the Mega Drive's A / B / C, L1 / X / R1 are X / Y / Z and SELECT is Mode; games made for the 6-button pad get one. SELECT + X switches between original and full screen. HOME → Screen → Mega Drive has Shader and Overlay (put your own 1024×768 PNGs in `Slot/Overlays/MD`; none come with Slot).
- HOME → Cart Art → GET CART ART fetches Mega Drive labels, and RetroAchievements work for Mega Drive games.
- Cart shape and picture size choices are kept per system: a Mega Drive game and a SNES game with the same name no longer share them. Choices made with older Slots are still read: a line under the bare name still counts for that game until you change it in Slot. A game moved between the GB and GBC folders no longer brings its choice with it.
- Fixed: SELECT + Y no longer changes the Game Boy colour correction while a PlayStation game is playing.
- Zipped games: put `.zip` or `.7z` ROMs in `Slot/Games/GB`, `GBC`, `GBA`, `NES`, `SNES` or `MD` and they play like the plain ROM, under the archive's name. Zip a ROM under its own name and it keeps its saves, states, label and play time. Each archive needs exactly one ROM in it; it is unpacked into `Slot/System/Cache/Unpacked` when you start it, and only the last one is kept. PlayStation games stay as they are.
- Pause menu: tap HOME in a game to pause it and choose Resume, Save State, Load State (opens the save state switcher), Reset Game or Quit Game. Reset Game starts the game over, as the console's reset button does: its battery save stays, play since your last save is lost. The battery, Wi-Fi and clock show along the bottom, as on the shelf. B or HOME resumes. Hold HOME and double-tap HOME work as before. Not during multiplayer, where the game can't pause.
- PlayStation (experimental, not yet tested on a Brick): put games in `Slot/Games/PS1` (`.cue`/`.bin`, `.chd`, `.pbp`, `.iso`, or an `.m3u` for a game of several discs) and they get a shelf of their own, as discs. The slot turns into a disc slot, and the disc spins as it goes in. They run on PCSX ReARMed, with your BIOS from `Slot/BIOS` if you have one.
- PlayStation games of several discs: A fans the discs out; pick one with left / right and A puts it in. To change discs mid-game, eject and put in the next one: the game carries on with it.
- PlayStation: L2 and R2 are the game's own buttons, so fast-forward is HOME + R2. No rewind for PlayStation games. A Brick Pro's left stick is the DualShock's left stick in game. Memory cards are saved in `Slot/Saves/PS1`, one per game. HOME → Screen → PlayStation has Shader, Dithering and Overlay.
- Cart Art fetches PlayStation disc scans, and RetroAchievements work for PlayStation games (`.chd` too).
- Title card: when you pick a game, its name shows on a card under its cart for a moment, then fades. Turn it off under HOME → Library → Title Card.
- Play time: how long you have played the selected game shows beside the battery on the shelf, in minutes ("200M") or in hours and minutes ("3H20M", HOME → Library → Play Time).
- Favorites: Y stars a game (a gold star on its cart), and HOME → Library → Show: Favorites shows only those.
- Sort the shelves with SELECT, or under HOME → Library → Sort By: A–Z, last played, or most played.
- Brick Pro (experimental, not yet tested on one): Slot recognises it, its left stick works as the D-pad, and System → LEDs has a Sticks row for the lights around its sticks.

## v0.44
**New since v0.43:**
- HOME → Update: update Slot over Wi-Fi. It shows what's new (L1 / R1 scroll long notes), asks before installing, and leaves your games, saves, states, labels and settings alone. Updates are signed and checked before anything is replaced; if the new version doesn't start, the previous one is put back, and an update cut short by the Brick switching off is cleared away. Needs 20% battery or a charger.
- HOME → System: LEDs, Date & Time and About are now under System, leaving the menu shorter.
- System → Auto Clock: with Wi-Fi on, the clock now sets itself from an internet time server, on connecting and every six hours. On by default; the time zone is still set under Date & Time.

## v0.43
**New since v0.42:**
- Overlays for every system come with Slot: KrutzOtrem's *Vanilla* set for GB, GBC and GBA and *CRT* set for NES and SNES, made for the Brick. Thanks to @KrutzOtrem!
- Overlays with a grid or tint over the game (such as KrutzOtrem's LCD and DMG grids) now work: the game fills the part of the overlay that isn't fully opaque, at the size the overlay was drawn for.
- An overlay name too long for its row is shortened, and scrolls slowly while the row is selected.
- Gameplay → Auto Save: save the game every 1, 5 or 10 minutes, or never. It was every minute.
- Gameplay → Low Battery Shutdown: save and shut down at 3%, 5% or 10% battery, or never. It was 5%.

## v0.42
**New since v0.41:**
- Screen settings are sorted by system: Game Boy / GBC, Game Boy Advance, NES and SNES each have their own page. NES and SNES shaders can now be set there too.
- Colour correction is set separately for the Game Boys and the GBA. A setting saved before keeps working for both.
- Overlays: put your own 1024×768 PNG overlays in `Slot/Overlays/<system>` and pick one on the system's Screen page. The game fits itself into the overlay's window. In game, HOME + L1 / R1 switches between them.

## v0.41
**New since v0.40:**
- Choosing a label now ends with it peeling onto the cart like a sticker, on every cart. On GBA carts the front closes for it, then opens again.
- Low-battery warning on the F1/F2 lights: steady red at 15% or below, blinking at 10% or below, even with the lights turned off. Thanks to @wargenng!
- Fixed: the About screen showed the wrong version.

## v0.40
**New since v0.23:**
- Wifi status icon on the shelf footer. Icons may change.
- Label Picker. Press Y when in the edit screen on a game cart. It has a label editor built in, so you can resize the label and make it fit the cart cutout
- Finally fixed Wi-Fi
- Customize cartridges: color, finish and shape for all shelves.
- Add SNES support: SNES and PAL cartridges
- Add NES support: NES and FAMICOM Cartridges
- Add RetroAchievements support, softcore only
- Automatic label fetcher in main menu, it fetches labels for all carts at once, but it is really barebones, so refer to the label picker for more control

**FOR DEVELOPERS**
- The Brick simulator - for testing in the browser before real hardware testing. Refer to the Development part of the Readme.

## v0.23
- Settings > About credits the port to LordNordyx.
- Short app names in the stock Apps list: Slot, Logo, Check and Stock, each with its own icon.
  Folder names are unchanged, so existing Apps/BootLogo/Image pictures keep working.
- Card README rewritten with the Brick's actual button layout (HOME = left button, SELECT = middle).
- Removed leftovers: the old Tools/tg5040/Slot.pak launcher and the RetroAchievements research note.

## v0.22.2
- Fixed "works once, then stock menu": the startup hook now waits until the real SD card is
  mounted instead of reading a stray Slot folder on internal storage.
- Check lists files hidden under the card mount on internal storage.

## v0.22.1
- The startup hook waits up to 30 s (was 10 s) for slow-mounting cards and logs each decision
  to Slot/Logs/hook.log.

## v0.22
- A read-only or damaged card no longer drops silently to the stock menu; Slot shows
  "Can't write to SD card".
- Check writes startup-report.txt at the top of the card.

## v0.21
- Fixed autostart failing after a firmware update or reinstall.
- New Check app: read-only startup report.

## v0.17 – v0.20
- SELECT + X switches GBA between 3:2 and 4:3 full screen, remembered per game.
- About screen describes the Brick port.

## v0.16
- Waking no longer needs two Power presses; Wi-Fi reconnects in the background.

## v0.15
- Merged upstream Slot 1.6.0: GB cartridge clocks kept in saves, read-only card alarm, faster
  library loading, shuffled wallpapers, crash logs.
- Boot Logo app (now Logo) replaces the separate install/restore logo apps.

## v0.14
- Lower sleep drain: radios off during suspend, real suspend-to-RAM where available.

## v0.13
- Auto-sleep after 5 min idle in menus or 15 min in game (Slot/DISABLE_AUTO_SLEEP turns it off).

## v0.12
- Wi-Fi works on stock firmware; minimum CPU clock in games fixes crackling audio.

Earlier versions: Brick screen, input, LEDs, power menu and autostart (see SOURCES.md).
