use std::path::{Path, PathBuf};

use slot_retro::{LibretroCore, MockCore, RetroCore};
use slot_store::{Core, GbPalette, Platform};

use crate::root;

const CORE_ENV: &str = "SLOT_CORE";

pub fn dylib_name(core: Core) -> String {
    format!(
        "{}_libretro.{}",
        core.as_str(),
        std::env::consts::DLL_EXTENSION
    )
}

fn candidates(root: &Path, core: Core) -> Vec<PathBuf> {
    if let Some(named) = std::env::var_os(CORE_ENV) {
        return vec![PathBuf::from(named)];
    }
    let name = dylib_name(core);
    let mut paths = vec![root.join("System").join(&name)];
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        paths.push(dir.join(&name));
        paths.push(dir.join("vendor").join(&name));
    }
    paths.push(Path::new("vendor").join(&name));
    paths
}

pub struct Opened {
    pub core: Box<dyn RetroCore>,
    pub named: bool,
}

pub fn open_core(
    root: &Path,
    core: Core,
    serial: &str,
    colour: bool,
    palette: Option<GbPalette>,
    link: Option<u8>,
) -> Opened {
    let paths = candidates(root, core);
    match open_named(root, core, serial, colour, palette, link, &paths, None) {
        Some(core) => Opened { core, named: true },
        None => {
            report_missing(core, &paths);
            Opened {
                core: Box::new(MockCore::new()),
                named: false,
            }
        }
    }
}

pub fn open_core_for(
    root: &Path,
    core: Core,
    serial: &str,
    colour: bool,
    paths: &[PathBuf],
) -> Box<dyn RetroCore> {
    open_core_for_palette(root, core, serial, colour, None, paths)
}

pub fn open_core_for_palette(
    root: &Path,
    core: Core,
    serial: &str,
    colour: bool,
    palette: Option<GbPalette>,
    paths: &[PathBuf],
) -> Box<dyn RetroCore> {
    open_named(root, core, serial, colour, palette, None, paths, None).unwrap_or_else(|| {
        report_missing(core, paths);
        Box::new(MockCore::new())
    })
}

/// The same bundled core as Mega Drive, with a handheld canvas and no width expansion.
pub fn open_game_gear(root: &Path) -> Opened {
    open_sega_8bit(root, Platform::Gg)
}

pub fn open_master_system(root: &Path) -> Opened {
    open_sega_8bit(root, Platform::Sms)
}

pub fn open_sega_cd(root: &Path) -> Opened {
    let paths = candidates(root, Core::PicoDrive);
    match open_named(root, Core::PicoDrive, "", false, None, None, &paths, Some(Platform::SegaCd)) {
        Some(core) => Opened { core, named: true },
        None => {
            report_missing(Core::PicoDrive, &paths);
            Opened { core: Box::new(MockCore::new()), named: false }
        }
    }
}

pub fn open_pce(root: &Path, platform: Platform) -> Opened {
    let paths = candidates(root, Core::BeetlePceFast);
    match open_named(root, Core::BeetlePceFast, "", false, None, None, &paths, Some(platform)) {
        Some(core) => Opened { core, named: true },
        None => {
            report_missing(Core::BeetlePceFast, &paths);
            Opened { core: Box::new(MockCore::new()), named: false }
        }
    }
}

fn open_sega_8bit(root: &Path, platform: Platform) -> Opened {
    let core = Core::GenesisPlusGx;
    let paths = candidates(root, core);
    match open_named(root, core, "", false, None, None, &paths, Some(platform)) {
        Some(core) => Opened { core, named: true },
        None => {
            report_missing(core, &paths);
            Opened {
                core: Box::new(MockCore::new()),
                named: false,
            }
        }
    }
}

fn open_named(
    root: &Path,
    core: Core,
    serial: &str,
    colour: bool,
    palette: Option<GbPalette>,
    link: Option<u8>,
    paths: &[PathBuf],
    game_platform: Option<Platform>,
) -> Option<Box<dyn RetroCore>> {
    if matches!(core, Core::Pico8 | Core::PortMaster) {
        return None;
    }
    let bios = root::bios_dir(root);
    let saves = if core == Core::FbNeo {
        root.join("Saves/NEOGEO")
    } else if core == Core::BeetleWswan {
        root.join("Saves/WS")
    } else if core == Core::BeetleLynx {
        root.join("Saves/LYNX")
    } else if core == Core::BeetleNgp {
        root.join("Saves/NGP")
    } else {
        root::saves_dir(root)
    };
    let _ = std::fs::create_dir_all(&saves);
    for path in paths {
        if !path.exists() {
            continue;
        }
        match LibretroCore::open_with(path, &bios, &saves) {
            Ok(mut opened) => {
                let platform = match core {
                    Core::Fceumm => Some(Platform::Nes),
                    Core::Snes9x => Some(Platform::Snes),
                    Core::PcsxRearmed => Some(Platform::Ps1),
                    Core::GenesisPlusGx => Some(game_platform.unwrap_or(Platform::Md)),
                    Core::ParallelN64 => Some(Platform::N64),
                    Core::PicoDrive => Some(game_platform.unwrap_or(Platform::S32x)),
                    Core::BeetlePceFast => Some(game_platform.unwrap_or(Platform::Pce)),
                    Core::FbNeo => Some(Platform::NeoGeo),
                    Core::BeetleNgp => Some(Platform::Ngp),
                    Core::BeetleWswan => Some(Platform::Ws),
                    Core::BeetleLynx => Some(Platform::Lynx),
                    Core::Pico8 | Core::PortMaster => return None,
                    Core::Mgba | Core::Gpsp => None,
                };
                if let Some((w, h)) = platform.map(Platform::canvas) {
                    opened.set_canvas(w, h);
                }
                opened.set_rotating_handheld(matches!(core, Core::BeetleWswan | Core::BeetleLynx));
                if core == Core::ParallelN64 {
                    opened.set_n64_controls();
                }
                // SMS 224/240-line modes fit the same 4:3 picture without clipping.
                opened.set_fill(
                    matches!(core, Core::PcsxRearmed | Core::ParallelN64)
                        || platform == Some(Platform::Sms)
                        || matches!(platform, Some(Platform::Pce | Platform::PceCd)),
                );
                opened
                    .set_fill_width((core == Core::GenesisPlusGx && platform == Some(Platform::Md)) || matches!(core, Core::PicoDrive | Core::FbNeo));
                // A DualShock: what DualShock games want, what plain ones run on as well, and
                // what carries rumble.
                if core == Core::PcsxRearmed {
                    opened.set_device(slot_retro::DEVICE_DUALSHOCK);
                }
                apply_core_options(
                    &mut opened,
                    core,
                    serial,
                    root::has_real_bios(root),
                    colour,
                    palette,
                );
                if let Some(player) = link {
                    apply_link_options(&mut opened, core, player);
                }
                eprintln!("slot: core {}", path.display());
                return Some(Box::new(opened));
            }
            Err(e) => eprintln!("slot: {}: {e}", path.display()),
        }
    }
    None
}

pub fn palette_for(
    chosen: Option<GbPalette>,
    platform: Platform,
    rom: &Path,
    link: Option<u8>,
) -> Option<GbPalette> {
    let game_boy = matches!(platform, Platform::Gb | Platform::Gbc);
    chosen.filter(|_| link.is_none() && game_boy && dmg_only(rom))
}

pub fn dmg_only(rom: &Path) -> bool {
    let mut header = [0u8; 0x150];
    slot_store::archive::read_at(rom, 0, &mut header).is_ok_and(|_| header[0x143] & 0x80 == 0)
}

fn report_missing(core: Core, paths: &[PathBuf]) {
    eprintln!(
        "slot: no {} core found, running the mock test pattern instead. Looked in: {}",
        core.as_str(),
        paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

pub fn colour_option(which: Core, on: bool) -> Option<(&'static str, &'static str)> {
    match which {
        Core::Mgba => Some(("mgba_color_correction", if on { "Auto" } else { "OFF" })),
        Core::Gpsp => Some((
            "gpsp_color_correction",
            if on { "enabled" } else { "disabled" },
        )),
        // A NES, SNES, Mega Drive or PlayStation drew on a TV, with no LCD to correct for.
        Core::Fceumm
        | Core::Snes9x
        | Core::PcsxRearmed
        | Core::GenesisPlusGx
        | Core::ParallelN64
        | Core::BeetleNgp
        | Core::BeetleWswan
        | Core::BeetleLynx
        | Core::PicoDrive
        | Core::BeetlePceFast
        | Core::FbNeo
        | Core::Pico8 | Core::PortMaster => None,
    }
}

/// PCSX ReARMed's dithering option, on or off.
pub fn dithering(on: bool) -> &'static str {
    match on {
        true => "enabled",
        false => "disabled",
    }
}

pub fn apply_link_options(core: &mut LibretroCore, which: Core, player: u8) {
    if which != Core::Mgba {
        return;
    }
    core.set_option("mgba_link", "on");
    core.set_option("mgba_link_player", &player.to_string());
    eprintln!("slot: core: link mode on, player {player}");
}

pub fn apply_core_options(
    core: &mut LibretroCore,
    which: Core,
    serial: &str,
    bios: bool,
    colour: bool,
    palette: Option<GbPalette>,
) {
    // FCEUmm and snes9x have no frameskip option; slot's own skipping covers them.
    if matches!(which, Core::Mgba | Core::Gpsp | Core::GenesisPlusGx) {
        core.set_option(&format!("{}_frameskip", which.as_str()), "auto");
    }
    if which == Core::BeetleLynx { core.set_option("lynx_rot_screen", "manual"); }
    if which == Core::BeetleWswan {
        core.set_option("wswan_rotate_display", "manual");
        core.set_option("wswan_rotate_keymap", "auto");
    }
    if which == Core::ParallelN64 {
        for (key, value) in [
            ("parallel-n64-gfxplugin", "rice"),
            ("parallel-n64-alt-map", "enabled"),
            ("parallel-n64-rspplugin", "hle"),
            ("parallel-n64-cpucore", "dynamic_recompiler"),
            ("parallel-n64-screensize", "320x240"),
            ("parallel-n64-filtering", "automatic"),
            ("parallel-n64-astick-deadzone", "0"),
            ("parallel-n64-astick-sensitivity", "100"),
            ("parallel-n64-pak1", "memory"),
        ] {
            core.set_option(key, value);
        }
    }
    if which == Core::Snes9x {
        // A 256x224 picture like the NES's, and no HD Mode 7: it would cost the A53 dearly
        // and send frames twice as wide for nothing on this screen.
        for (key, value) in [("snes9x_overscan", "enabled"), ("snes9x_mode7_hires", "disabled")] {
            core.set_option(key, value);
        }
    }
    if which == Core::GenesisPlusGx {
        // The region the header names, no TMSS BIOS screen, no border round the picture, and
        // interlaced games kept at 224 lines.
        // Port 1 stays a plain joypad: the core then gives a game a 6-button pad when its
        // header says it takes one and a 3-button pad otherwise, as some older games misread
        // a 6-button pad.
        for (key, value) in [
            ("genesis_plus_gx_region_detect", "auto"),
            ("genesis_plus_gx_bios", "disabled"),
            ("genesis_plus_gx_overscan", "disabled"),
            ("genesis_plus_gx_gg_extra", "disabled"),
            ("genesis_plus_gx_render", "single field"),
        ] {
            core.set_option(key, value);
        }
    }
    if which == Core::FbNeo {
        for (key, value) in [
            ("fbneo-neogeo-mode", "MVS_EUR"),
            ("fbneo-memcard-mode", "per-game"),
            ("fbneo-diagnostic-input", "None"),
            ("fbneo-hiscores", "disabled"),
            ("fbneo-samplerate", "44100"),
        ] { core.set_option(key, value); }
    }
    if which == Core::BeetlePceFast {
        for (key, value) in [
            ("pce_fast_cdbios", "System Card 3"),
            ("pce_fast_cdspeed", "1"),
            ("pce_fast_default_joypad_type_p1", "2 Buttons"),
            ("pce_fast_turbo_toggling", "disabled"),
            ("pce_fast_frameskip", "disabled"),
            ("pce_fast_hoverscan", "352"),
            ("pce_fast_initial_scanline", "3"),
            ("pce_fast_last_scanline", "242"),
        ] { core.set_option(key, value); }
    }
    if which == Core::PicoDrive {
        for (key, value) in [
            ("picodrive_region", "Auto"),
            ("picodrive_input1", "6 button pad"),
            ("picodrive_input2", "None"),
            // Preserve the console's internal backup RAM through libretro SAVE_RAM.
            ("picodrive_ramcart", "disabled"),
            ("picodrive_drc", "enabled"),
            ("picodrive_renderer", "accurate"),
            ("picodrive_frameskip", "disabled"),
        ] {
            core.set_option(key, value);
        }
    }
    if which == Core::Fceumm {
        // 8 lines off the top and bottom, as a TV hid them: the 256x224 Platform::Nes draws.
        for (key, value) in [
            ("fceumm_overscan_v_top", "8"),
            ("fceumm_overscan_v_bottom", "8"),
            ("fceumm_overscan_h_left", "0"),
            ("fceumm_overscan_h_right", "0"),
        ] {
            core.set_option(key, value);
        }
    }
    if which == Core::PcsxRearmed {
        for (key, value) in [
            // The player's BIOS from Slot/BIOS when there is one, the core's own otherwise.
            ("pcsx_rearmed_bios", "auto"),
            ("pcsx_rearmed_show_bios_bootlogo", "disabled"),
            // Memory card 1 is the game's save RAM, kept in Saves/PS1 like a cart's battery save.
            ("pcsx_rearmed_memcard1", "libretro"),
            ("pcsx_rearmed_frameskip_type", "auto"),
            ("pcsx_rearmed_vibration", "enabled"),
        ] {
            core.set_option(key, value);
        }
    }
    if which == Core::Mgba {
        core.set_option("mgba_sgb_borders", "OFF");
        core.set_option("mgba_force_gbp", "ON");
        match palette {
            Some(p) => {
                core.set_option("mgba_gb_model", "Game Boy");
                core.set_option("mgba_gb_colors_preset", "0");
                core.set_option("mgba_gb_colors", p.core_name());
            }
            None => {
                core.set_option("mgba_gb_model", "Autodetect");
                core.set_option("mgba_gb_colors_preset", "1");
                core.set_option("mgba_gb_colors", "GBC Dark Green →A");
            }
        }
        if let Some((key, value)) = colour_option(which, colour) {
            core.set_option(key, value);
        }
    }
    if which == Core::Gpsp {
        core.set_option("gpsp_serial", serial);
        core.set_option("gpsp_rtc_time_source", "system");
        if bios {
            core.set_option("gpsp_boot_mode", "bios");
        }
        if let Some((key, value)) = colour_option(which, colour) {
            core.set_option(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn each_core_spells_colour_correction_its_own_way() {
        assert_eq!(
            colour_option(Core::Mgba, true),
            Some(("mgba_color_correction", "Auto"))
        );
        assert_eq!(
            colour_option(Core::Mgba, false),
            Some(("mgba_color_correction", "OFF"))
        );
        assert_eq!(
            colour_option(Core::Gpsp, true),
            Some(("gpsp_color_correction", "enabled"))
        );
        assert_eq!(
            colour_option(Core::Gpsp, false),
            Some(("gpsp_color_correction", "disabled"))
        );
    }

    fn lock() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn candidates_search_the_named_cores_own_filename_only() {
        let _g = lock();
        std::env::remove_var(CORE_ENV);

        let root = Path::new("/root");
        let mgba = candidates(root, Core::Mgba);
        let gpsp = candidates(root, Core::Gpsp);
        assert_ne!(mgba, gpsp, "the two cores searched the same paths");
        assert!(!mgba.is_empty());
        assert!(!gpsp.is_empty());
        for path in &mgba {
            let name = path.file_name().unwrap().to_str().unwrap();
            assert!(name.starts_with("mgba_libretro"), "{name} is not mGBA's");
        }
        for path in &gpsp {
            let name = path.file_name().unwrap().to_str().unwrap();
            assert!(name.starts_with("gpsp_libretro"), "{name} is not gpSP's");
        }
    }

    #[test]
    fn candidates_search_the_roots_own_system_directory() {
        let _g = lock();
        std::env::remove_var(CORE_ENV);
        let root = Path::new("/some/content/root");
        assert_eq!(
            candidates(root, Core::Gpsp)[0],
            root.join("System").join(dylib_name(Core::Gpsp)),
        );
    }

    #[test]
    fn the_env_override_ignores_which_core_was_asked_for() {
        let _g = lock();
        std::env::set_var(CORE_ENV, "/dev/null/named-core");
        let root = Path::new("/root");
        let mgba = candidates(root, Core::Mgba);
        let gpsp = candidates(root, Core::Gpsp);
        std::env::remove_var(CORE_ENV);
        assert_eq!(mgba, vec![PathBuf::from("/dev/null/named-core")]);
        assert_eq!(
            mgba, gpsp,
            "the override stopped winning for one of the cores"
        );
    }
}
