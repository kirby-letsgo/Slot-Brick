# Development roadmap

This is a working backlog, not a promise of performance or a release date.
Hardware capabilities must be verified on actual TrimUI devices, not inferred
from simulator results.

## 1. Per-game emulator settings (next)

Give each emulated game its own settings, reachable from the cartridge's
options **and** from **HOME > Pause > Emulator Settings**. Keep existing
per-system/global defaults; only explicitly chosen game overrides should
replace them. Show the effective values and offer **Restore defaults for this
game**.

- [ ] **FPS counter (priority):** toggle an unobtrusive in-game overlay per
  game and remember it across launches. Measure real emulator frames per
  elapsed second using the existing emulator-published-frame counter, rather
  than simply reporting screen redraw rate. Update at a readable cadence
  (e.g. 500–1000 ms), show FPS only during gameplay, and exclude paused
  periods from averages. Document fast-forward and frame-skipping semantics.
- [ ] Add Integer Scale for each emulated game: the largest whole-number enlargement that fits the physical panel, centred without cropping, and with the existing Actual/Stretch preference kept when disabled.
- [ ] Surface meaningful existing options (depending on system/core): shader,
  fast-forward speed/sound, rewind where supported, rumble, and applicable
  colour correction or PlayStation dithering.
- [ ] Distinguish **System default** from a per-game override; editing one
  game must not alter another game's settings or the global Slot state.
- [ ] Make the pause menu work without unpausing/resetting the core and make
  per-game changes effective immediately wherever the backend supports that.
  If a change needs a core restart, explicitly explain that in the menu.
- [ ] Persist settings atomically under `Slot/Config` with stable per-game
  identity (platform plus ROM identity), survive updates, and handle deleted
  games safely.
- [ ] Hide unsupported settings for native PICO-8/Ports and system-specific
  cores. Don't offer rewind where it is disabled for safety/performance.
- [ ] Add unit, integration and simulator coverage; check frame pacing,
  returns to gameplay, save states, zero/empty games, and game-to-game
  isolation. Verify hardware behaviour on Brick, Hammer and Brick Pro.

## 2. Dreamcast emulation (after per-game settings and FPS)

- [ ] Investigate the Flycast libretro core and whether it builds, launches,
  and works with Slot's stock-firmware graphics, input, sound and filesystem
  stack on Brick / Hammer / experimental Brick Pro.
- [ ] Benchmark original/homebrew diagnostic content **on real devices**,
  including game FPS, audio, memory pressure, thermals and battery use.
  Evaluate whether performance is acceptable and whether the system should
  be experimental, model-gated, or omitted.
- [ ] Confirm supported game formats and optional/required user-provided
  BIOS, with accurate setup and compatibility documentation.
- [ ] If viable: add a Dreamcast system tile, disc/cart presentation,
  scanning, core selection and launch/return, controls (including analog
  mapping), saves/states, artwork and the new per-game settings.
- [ ] Add missing-BIOS/unplayable-game guidance, automated core smoke tests
  and hardware acceptance testing.

**Sequencing:** Complete and validate phase 1 before starting phase 2.
