# BIOS and runtime setup

For the [kirby-letsgo fork of Slot-Brick](https://github.com/kirby-letsgo/Slot-Brick).
Supply your own BIOS files and PICO-8 runtime; they are not included in the card download.
The libretro emulator cores are included; Dreamcast's optional standalone Flycast
runtime is installed separately.

All paths here start at the **SD card root**. Put BIOS files directly in **`Slot/BIOS/`**,
except for Dreamcast's files in **`Slot/BIOS/DC/`** and PICO-8's files in
**`Slot/BIOS/PICO8/`**. Use the filenames and
capitalization shown below. Only install files for the systems and regions you play.
These files survive Slot updates.

## Quick reference

| System | Required? | File in `Slot/BIOS/` | Size |
| :--- | :--- | :--- | :--- |
| Atari Lynx | Yes | `lynxboot.img` | 512 bytes |
| Sega Dreamcast | Yes | `DC/dc_boot.bin` | 2,097,152 bytes (2 MiB) |
| Sega CD / Mega-CD — USA | For USA games | `bios_CD_U.bin` | 131,072 bytes (128 KiB) |
| Sega CD / Mega-CD — Europe | For European games | `bios_CD_E.bin` | 131,072 bytes (128 KiB) |
| Sega CD / Mega-CD — Japan | For Japanese games | `bios_CD_J.bin` | 131,072 bytes (128 KiB) |
| PC Engine CD / TurboGrafx-CD | Yes | `syscard3.pce` | 262,144 bytes (256 KiB), or 262,656 with a copier header |
| Neo Geo | Yes | `neogeo.zip` — keep zipped | Varies; see archive requirements below |
| PlayStation | Optional, recommended | `scph5500.bin`, `scph5501.bin`, or `scph5502.bin` | 524,288 bytes (512 KiB) each |
| Game Boy Advance | Optional | `gba_bios.bin` | 16,384 bytes (16 KiB) |
| PICO-8 | Official runtime required | `PICO8/pico8_64` **and** `PICO8/pico8.dat` | Varies by PICO-8 version |

**No external BIOS is required** for Game Boy, Game Boy Color, NES, SNES, Mega Drive,
Game Gear, Master System, Nintendo 64, Neo Geo Pocket / Color, Sega 32X, PC Engine
HuCards, or WonderSwan / Color. WonderSwan and Color games both go in `Slot/Games/WS`.
Neo Geo Pocket is separate from Neo Geo arcade and does not use `neogeo.zip`.

## Check your setup on the Brick

Open **HOME → System → System Setup**. Select a system and press **A** for details;
press **X** to recheck after changing files.

- **Ready:** the required setup files passed the checks.
- **Missing:** a required BIOS, runtime or emulator file was not found.
- **Check files:** a file needs attention, or a regional BIOS is absent. Read the details.
- **Optional:** the system can run without the suggested file, as with PlayStation's BIOS.

The checker examines file sizes, archive entries and emulator/runtime headers. It does
not launch games or verify every BIOS checksum. A file of the correct size can still
contain the wrong data, and **Ready** does not guarantee a game's compatibility.
For Sega CD, **Check files** can simply mean you have BIOS files for only some regions;
the details show which are installed. You only need the regions you play.

## Atari Lynx

Copy the **512-byte Lynx boot ROM** to:

```text
Slot/BIOS/lynxboot.img
```

Slot checks for this file before launching Lynx games. A missing or wrong-size file
shows setup instructions. Games go in `Slot/Games/LYNX` as `.lnx`, `.lyx` or homebrew
`.o` files, either plain or one ROM per ZIP/7z.

## Sega Dreamcast — experimental native Flycast

Copy your **2 MiB Dreamcast boot ROM** to:

```text
Slot/BIOS/DC/dc_boot.bin
```

The launcher requires this file and checks its size before starting Flycast. If your
Flycast build needs `dc_flash.bin`, put it beside `dc_boot.bin` in `Slot/BIOS/DC/`.
Slot's setup checker checks the boot ROM; it does not validate the optional flash file.

Dreamcast also needs the separately installed ARM64 Flycast runtime in
`Slot/System/Flycast/`. Follow the [Dreamcast setup guide](DREAMCAST.md) to install it
and review the experimental support and hardware limitations. A BIOS alone does not
install the emulator.

Games go in `Slot/Games/DC/` as CHD, GDI, CDI, CUE or M3U files. Keep referenced disc
tracks beside their GDI/CUE descriptor. Launch errors are logged in
`Slot/Logs/dreamcast.log` and `Slot/Logs/slot.log`.

## Sega CD / Mega-CD

Copy the **128 KiB BIOS for each game's region** directly into `Slot/BIOS` using the
names in the table. A USA BIOS does not replace the Japanese or European BIOS. Plain
`.bin` files with these canonical names are the simplest setup; do not put them in a
`SEGACD` subfolder or use a 32X BIOS instead.

PicoDrive also recognizes alternate BIOS names and ZIP versions, but this guide uses
the standard uncompressed filenames. Renaming a BIOS from another region does not
change its region or contents.

Games go in `Slot/Games/SEGACD` as CHD, CUE with all referenced tracks, ISO or M3U
playlists. ZIP/7z game discs are not supported. Keep the track files beside their CUE.

## PC Engine CD / TurboGrafx-CD

Copy your **System Card 3** BIOS to:

```text
Slot/BIOS/syscard3.pce
```

Slot accepts 262,144 bytes, or 262,656 bytes with a 512-byte copier header. An older
System Card renamed to `syscard3.pce` is not a replacement for System Card 3.

CD games go in `Slot/Games/PCECD` as CHD, CUE with its tracks or M3U playlists. Plain
HuCard games go in `Slot/Games/PCE` and do not need this BIOS.

## Neo Geo

Use a complete **FBNeo-compatible `neogeo.zip`** at:

```text
Slot/BIOS/neogeo.zip
```

A copy at `Slot/Games/NEOGEO/neogeo.zip` also works and is hidden from the shelf.
Keep the archive intact. Do not extract it into the BIOS folder, rename individual
chips, or wrap its contents in an extra directory inside the ZIP.

Slot uses **European MVS arcade mode** by default. System Setup looks for these
entries at the archive root:

| Entry | Expected size |
| :--- | :--- |
| A European MVS BIOS: `sp-s3.sp1`, `sp-s2.sp1` or `sp-s.sp1` | 131,072 bytes |
| `sm1.sm1` | 131,072 bytes |
| `sfix.sfix` | 131,072 bytes |
| `000-lo.lo` | 131,072 bytes |

These are the setup checker's minimum checks, not instructions to construct a new
archive. A complete set can contain additional BIOS variants. Use a BIOS archive
and game sets compatible with the bundled FinalBurn Neo core; FBNeo checks the actual
chip contents at launch. An archive with the right filenames can still be incompatible.

Game ZIP/7z files belong in `Slot/Games/NEOGEO`. Preserve original short names such as
`mslug.zip`. A non-merged set is simplest; split clone sets also need their parent
archive beside them. MAME/FBA sets may have different chips or checksums. If a game
returns to the shelf, check `Slot/Logs/slot.log` for missing or mismatched chips.

## PlayStation — optional, recommended

PCSX ReARMed can use built-in BIOS emulation. A real BIOS improves compatibility.
The standard files, each **512 KiB**, are:

| Region | Path |
| :--- | :--- |
| Japan | `Slot/BIOS/scph5500.bin` |
| USA | `Slot/BIOS/scph5501.bin` |
| Europe | `Slot/BIOS/scph5502.bin` |

Use the appropriate regional file; you can keep all three together. The setup checker
also recognizes 512 KiB files named `psxonpsp660.bin`, `scph101.bin`, `scph7001.bin`
and `scph1001.bin`. The emulator may recognize other names at launch. These files
must be uncompressed; `scph5501.bin.zip` is not the BIOS file itself.

Put games in `Slot/Games/PS1`. For CUE/BIN games, copy both the CUE and all its tracks.
CHD is also supported; multi-disc games can use an M3U playlist. A valid BIOS does not
repair an incomplete disc dump or a CUE pointing to missing files.

## Game Boy Advance — optional

GBA games can use the cores' built-in BIOS support. To supply the real BIOS, including
the boot screen, copy the **16 KiB** file to:

```text
Slot/BIOS/gba_bios.bin
```

The setup check also expects its first byte to be `0x18`; size alone is not enough.
The absence of this optional file does not prevent GBA games from launching.

## PICO-8 — official runtime and Splore

PICO-8 uses a paid native runtime rather than a console BIOS. From your own
[Lexaloffle account's download](https://www.lexaloffle.com/pico-8.php), extract the
**Raspberry Pi package** and copy these two files from the same download:

```text
Slot/BIOS/PICO8/pico8_64
Slot/BIOS/PICO8/pico8.dat
```

The launcher requires the **ARM64 `pico8_64`** executable. Windows, macOS, x86 Linux
and the 32-bit `pico8_dyn` executable will not work. Copy the files themselves into
`PICO8`, without an extra enclosing download folder. Slot supplies its compatibility
and download helpers in `Slot/System`; keep those from the Slot package.

Put `.p8.png`, `.png` or `.p8` games in `Slot/Games/PICO8`. PNG cartridges contain
both the game and its artwork, so no Cart Art download is needed.

**You do not need to create a file for Splore.** Its cartridge appears when the runtime
or a PICO-8 game is installed. Connect Wi-Fi in Slot, check the clock, and open Splore
to browse and download games. Return with **START → Options → shutdown PICO-8**.
PICO-8 uses its own saves; Slot's save states do not apply.

Runtime or download problems are logged in `Slot/Logs/pico8.log`. Keep the bundled
`Slot/System/pico8-bin/wget` helper when updating. Runtime files, native saves in
`Slot/Saves/PICO8` and settings/cache in `Slot/Config/PICO8` survive Slot updates.

## Copying and troubleshooting

You can copy files using a card reader or **HOME → Wi-Fi → File Transfer**. Use the
connection details shown on the Brick in your FTP client. Press **B** to stop transfers,
then recheck System Setup with **X**.

Check the following if a file is still reported missing:

1. The path starts with `Slot/BIOS`, not `BIOS` at the card root or a folder under
   `Slot/Games`. Dreamcast uses the extra `DC` subfolder; PICO-8 uses `PICO8`.
2. The filename matches exactly, including capital `CD` and the region letter in
   `bios_CD_U.bin`. Show file extensions in Finder or Explorer to catch names such as
   `lynxboot.img.img` or `scph5501.bin.zip`.
3. The file has the correct byte count and is the actual BIOS, not a shortcut, HTML
   download page or macOS `._` metadata file. On a Mac with a card named `SLOT`, for example:

   ```sh
   wc -c < "/Volumes/SLOT/Slot/BIOS/lynxboot.img"
   ```

   This should print `512`. Substitute the relevant path and compare against the table.
4. If System Setup reports an emulator or PICO-8 helper missing, restore that file from
   this fork's latest card package. A BIOS cannot replace a missing emulator core.

For launch failures after setup passes, read `Slot/Logs/slot.log` (or `pico8.log` for
PICO-8). Include your build, system, game format and relevant log when
[reporting a problem](https://github.com/kirby-letsgo/Slot-Brick/issues).

This guide is also installed as `Slot/System/BIOS.md` on the card for offline reference.
