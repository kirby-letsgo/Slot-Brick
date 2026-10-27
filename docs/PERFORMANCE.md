# Startup and library performance

Slot uses a cached game index to open the library, then checks for added or removed games in the background. The October 2026 performance pass also removes two sources of foreground work:

- Building shelves, opening search and restoring the shelves no longer open every ROM or decompress its header just to choose a cartridge shadow. Artwork workers resolve that shape for the cartridges being prepared. Prepared shapes and textures follow their games through search and sorting.
- Startup no longer fits and caches all newly added labels before accepting input. Visible and nearby cartridges get their artwork on background workers as you browse, using the existing disk cache. A frame waits at most 2 ms for worker results, shared across visible carts. Image upload and drawing have their own costs; this is a limit on waiting for workers, not a frame-time guarantee. Placeholders can appear briefly while an image loads.

These changes target frontend responsiveness. Emulator settings, CPU clocks and the firmware boot sequence are unchanged.

## Library benchmark

Measured on the same Linux x86-64 host with an optimized simulator build, comparing `de4db3c` with this performance pass. The fixture contains 4,000 ZIP files: 1,000 each for GB, NES, SNES and MD, with 64 KiB synthetic ROM headers. Values are medians of four warm runs after one warm-up. The library scan happens outside the timers; rendering, input latency and emulator startup are excluded.

| Operation | Before | After |
| --- | ---: | ---: |
| Construct library/shelves | 69.77 ms | 0.70 ms |
| Open all-game search | 74.40 ms | 3.59 ms |
| Close search / restore shelves | 71.34 ms | 1.53 ms |

These are host component timings, **not Brick power-on times**. An SD card has different I/O costs. The regression tests additionally use blocking FIFO files to verify that constructing and searching the library never opens the ROMs, independently of machine speed.

To reproduce from the repository root, choose a new temporary folder:

```sh
python3 brick/sim/make-perf-library.py /tmp/slot-perf/Slot
cargo run --locked --release -p slot --no-default-features --features sim \
  --example library_bench -- /tmp/slot-perf/Slot
```

The generator refuses an existing destination. It creates header-only test files, not playable games. Use `--labels 500` to include 500 synthetic PNG labels when testing startup. For a comparison against an earlier checkout, use the same fixture and benchmark source with both builds.

## Startup smoke measurement

With the same 4,000-game fixture plus 500 uncached 512×512 PNG labels, a simulator script took a median **1.73 s before / 1.18 s after** across three alternating runs. It waited for a stable clock screen, accepted A, and waited for a stable system grid. Both screenshots showed the expected screens. Each run cleared only the fixture's configuration and derived label cache; the library index was warm after the first run.

This measures the entire host process/script, including software rendering, two 400 ms settling waits and screenshot capture. It is a reproducible smoke measurement, not an estimate of physical boot time. The script was:

```text
phase SetClock
idle 400 60000
grab clock
press a
phase Shelf
idle 400 60000
grab grid
```

## Browsing I/O limits

ROM prefetch starts after 500 ms on the same selection, reads at most 4 MiB and skips CHD, CUE, ISO, PBP and M3U files. Files larger than 64 MiB are also skipped. File-size checks happen on the worker once per selection, keeping filesystem access off the input thread. Changing selection, leaving the shelf or dropping the prefetcher cancels obsolete reads between 256 KiB chunks; an operating-system read already in progress must still finish.

Cartridge artwork workers queue the selected cartridge first, followed by visible neighbours. The render thread never waits for results and uploads at most three completed faces per frame, retaining additional results for later frames. Regression tests cover stalled workers, burst limits, capped reads and cancellation; physical SD-card latency still needs measurement.

## Check on a Brick

Compare builds on the same device, SD card, library and settings. Record power-button-to-responsive-grid time across several cold boots, separately from waking from sleep. Also check opening search, scrolling quickly and returning from a game. Include a run after copying fresh labels to the card; the grid should accept input immediately while artwork is prepared as you browse.

Physical Brick timing has not yet been measured for this pass. Existing first-frontend-frame log timing helps isolate frontend startup, but does not include firmware startup and does not by itself prove that input is responsive.
