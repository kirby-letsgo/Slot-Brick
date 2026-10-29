# Dreamcast on Slot-Brick — experimental native Flycast

Dreamcast is an **opt-in experiment**, not a claim of playable performance
on stock TrimUI Brick firmware, Brick Hammer or Brick Pro. A standalone
Flycast binary is launched after Slot releases its audio, video and input
devices. It returns to the existing cartridge shelf when Flycast exits.
The Flycast binary comes from an independently maintained **MinUI**
Dreamcast package tested on TrimUI Brick; Slot-Brick **does not bundle it**
and its compatibility with the stock Slot environment needs hardware testing.

## Setup (computer and SD card)

1. Install a recent Slot-Brick Dev build first.
2. Mount your card on a computer and run
   `python3 brick/setup-dreamcast.py --card /path/to/SDCARD`
   from a checkout of this repository. This downloads and SHA-256 verifies the
   pinned [MinUI DC.pak v0.6.5](https://github.com/josegonzalez/minui-dreamcast-pak/releases/tag/0.6.5)
   and installs just the optional ARM64 Flycast binary, runtime libraries,
   configuration template, controller mapping and licences into
   `Slot/System/Flycast/`. Offline: download `DC.pak.zip` yourself and pass
   `--pak /path/to/DC.pak.zip`. Neither procedure modifies firmware.
3. Provide your own legally obtained Dreamcast BIOS:
   `Slot/BIOS/DC/dc_boot.bin` (2 MiB). No BIOS is distributed with Slot.
   `dc_flash.bin` can also be supplied if needed by your Flycast build.
4. Put games in `Slot/Games/DC/`. Supported shelf formats are `.chd`,
   `.gdi`, `.cdi`, `.cue`, and `.m3u`; each may live in one directly nested
   game folder. Keep GDI and CUE referenced tracks beside their descriptor.
   Loose `.bin` or `.raw` tracks are not listed as games.
5. Open the Dreamcast tile on Slot's system grid. Select a disc and press A.
   For installation checks, open HOME → System → System Setup → Dreamcast.

The pinned release version and SHA-256 are in `brick/setup-dreamcast.py`.
Source licences and original metadata are kept under `Slot/System/licenses/`
and the package's upstream repository. Please respect both the emulator
and game-data licences.

## Controls, FPS and saves

- The standalone Flycast process has its own input mapping and settings UI,
  independent of Slot's libretro emulators. The initial mapping shipped by the
  upstream package targets the stock controller's SDL joystick. You can edit
  `Slot/Config/DC/mappings/SDL_Xbox 360 Controller.cfg`. Physical button/axis
  mappings need verification on the particular Brick model. Slot corrects the
  upstream Select/HOME assignments at launch: **HOME/Menu opens Flycast's menu**
  and Select no longer opens settings. Existing mappings are migrated too;
  the other gameplay bindings are preserved.
- On devices without an analog stick, Dreamcast games needing analog movement
  may be difficult to control. Some games cannot be played fully with the D-pad.
- The native Flycast image gets the **entire screen**, not Slot's usual pause
  menu. Use Flycast's own in-game menu for VMU, resolution and native settings;
  HOME held exits through Slot's native hand-off, then reopens the shelf.
  Power pauses the native process and turns the backlight off.
- Slot's native SDL audio bridge applies the saved speaker/headphone volume,
  volume buttons and mute while Flycast runs. Keep
  `Slot/System/pico8-sdl-nosensor.so` from the card package installed and use
  Flycast's SDL2 audio backend.
- Use the cartridge's **START → Emulator Settings → SHOW FPS** to toggle
  **Flycast's own** performance display for that game. The choice is saved
  per-game in `Config/game-settings.txt` and applied at the *next launch*.
  Shader, rewind, integer scaling, save states and Slot's FPS overlay do not
  operate inside the native Flycast process.
- Flycast writes configuration and mappings under `Slot/Config/DC/` and its
  standalone VMU/save-state files under `Slot/Saves/DC/`. These are not Slot's
  regular libretro savestates. Back up both directories. A separate
  `Slot/Logs/dreamcast.log` records process output and errors.
- **F1 saves a state; F2 loads it** by default, on both Brick and Brick Pro.
  These use Flycast's currently selected state slot. Slot's **System → Controls**
  assignments for F1/F2 in a game are applied when Dreamcast next launches:
  Save State, Load State and Off are supported. Menu stays on HOME/Menu.
  Flycast accepts one button per action, so assigning the same save/load action
  to both function keys uses F1. Other assignments leave that function key
  unbound in Flycast. Choose the state slot in Flycast's menu;
  these shortcuts do not use Slot's save-state switcher or backups.

## Feasibility and limitations

Dreamcast holds the CPU at the highest frequency in Slot's verified table,
within the firmware's existing maximum and Slot's 1.8 GHz ceiling. This prevents
the governor dropping the clock between Flycast's bursts of work. Power-button
sleep releases the held frequency; exiting restores the original CPU policy.
This uses more battery than allowing the CPU to downclock. It does not change
voltage, GPU clocks or the firmware's frequency limits. `Slot/DISABLE_POWER_SAVING`
also disables this CPU management.

For demanding games such as Crazy Taxi, enable the FPS counter and compare the
same part of a race before and after updating. If it still runs slowly, try
Flycast's Video settings one at a time: Internal Resolution **320×240 (Half)**,
then **Delay Frame Swapping off**. Half resolution reduces GPU work at the cost
of a softer image; disabling delayed swaps can introduce visual glitches.
Keep threaded rendering enabled and widescreen hacks off while comparing.
Slot preserves your Flycast graphics settings instead of forcing these choices.

The available binary was tested by its maintainer with **MinUI** on TrimUI
Brick; its SDL, EGL/GLES, library/driver and CPU behaviour under **stock**
firmware are *not verified*. Full speed is not guaranteed, especially for
3D games. Only a real device can establish FPS, input, audio glitches, heat,
battery draw and whether Flycast launches at all. Native launches will
leave the frontend briefly and are **not compatible with background FTP**.

Flycast may require functionality absent from the current firmware or
additional version-specific runtime dependencies. If setup or boot fails,
Slot shows an error after returning to the shelf. Attach
`Slot/Logs/dreamcast.log` and `Slot/Logs/slot.log` when reporting issues.
The installer never patches the OS, downloads BIOS/ROMs, or overwrites
existing games, saves or settings.

Future work after hardware tests: verify the stock controller mapping,
model-specific compatibility and GPU context; consider different renderer
versions or a libretro backend only if needed and demonstrably stable.
