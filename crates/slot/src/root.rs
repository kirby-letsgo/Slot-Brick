use std::path::{Path, PathBuf};

/// Every platform in Platform::ALL has its five folders here (tests/dist.rs checks).
pub const DIRS: [&str; 106] = [
    "BIOS",
    "BIOS/PICO8",
    "Config",
    "Config/PICO8",
    "Games",
    "Games/GBA",
    "Games/GB",
    "Games/GBC",
    "Games/NES",
    "Games/SNES",
    "Games/MD",
    "Games/PS1",
    "Games/GG",
    "Games/SMS",
    "Games/N64",
    "Games/NGP",
    "Games/WS",
    "Games/LYNX",
    "Games/PICO8",
    "Games/32X",
    "Games/SEGACD",
    "Games/PCE",
    "Games/PCECD",
    "Games/NEOGEO",
    "Labels",
    "Labels/GBA",
    "Labels/GB",
    "Labels/GBC",
    "Labels/NES",
    "Labels/SNES",
    "Labels/MD",
    "Labels/PS1",
    "Labels/GG",
    "Labels/SMS",
    "Labels/N64",
    "Labels/NGP",
    "Labels/WS",
    "Labels/LYNX",
    "Labels/PICO8",
    "Labels/32X",
    "Labels/SEGACD",
    "Labels/PCE",
    "Labels/PCECD",
    "Labels/NEOGEO",
    "Overlays",
    "Overlays/GBA",
    "Overlays/GB",
    "Overlays/GBC",
    "Overlays/NES",
    "Overlays/SNES",
    "Overlays/MD",
    "Overlays/PS1",
    "Overlays/GG",
    "Overlays/SMS",
    "Overlays/N64",
    "Overlays/NGP",
    "Overlays/WS",
    "Overlays/LYNX",
    "Overlays/PICO8",
    "Overlays/32X",
    "Overlays/SEGACD",
    "Overlays/PCE",
    "Overlays/PCECD",
    "Overlays/NEOGEO",
    "Saves",
    "Saves/GBA",
    "Saves/GB",
    "Saves/GBC",
    "Saves/NES",
    "Saves/SNES",
    "Saves/MD",
    "Saves/PS1",
    "Saves/GG",
    "Saves/SMS",
    "Saves/N64",
    "Saves/NGP",
    "Saves/WS",
    "Saves/LYNX",
    "Saves/PICO8",
    "Saves/32X",
    "Saves/SEGACD",
    "Saves/PCE",
    "Saves/PCECD",
    "Saves/NEOGEO",
    "States",
    "States/GBA",
    "States/GB",
    "States/GBC",
    "States/NES",
    "States/SNES",
    "States/MD",
    "States/PS1",
    "States/GG",
    "States/SMS",
    "States/N64",
    "States/NGP",
    "States/WS",
    "States/LYNX",
    "States/PICO8",
    "States/32X",
    "States/SEGACD",
    "States/PCE",
    "States/PCECD",
    "States/NEOGEO",
    "System",
    "Wallpapers",
];

pub fn ensure(root: &Path) {
    for sub in DIRS {
        let _ = std::fs::create_dir_all(root.join(sub));
    }
    if let Err(e) = slot_store::move_config(root) {
        eprintln!("slot: config: {e}");
    }
}

pub fn bios_dir(root: &Path) -> PathBuf {
    root.join("BIOS")
}

const BIOS_FILE: &str = "gba_bios.bin";

const BIOS_BYTES: u64 = 16 * 1024;
const BIOS_FIRST_BYTE: u8 = 0x18;

pub fn has_real_bios(root: &Path) -> bool {
    let Ok(mut f) = std::fs::File::open(bios_dir(root).join(BIOS_FILE)) else {
        return false;
    };
    if !f.metadata().is_ok_and(|m| m.len() == BIOS_BYTES) {
        return false;
    }
    let mut first = [0u8; 1];
    std::io::Read::read_exact(&mut f, &mut first).is_ok() && first[0] == BIOS_FIRST_BYTE
}

pub fn saves_dir(root: &Path) -> PathBuf {
    root.join("Saves")
}

/// The Lynx core cannot boot normal carts without its 512-byte boot ROM.
pub fn check_lynx_bios(root: &Path) -> Result<(), &'static str> {
    if root.join("BIOS/lynxboot.img").metadata().is_ok_and(|m| m.is_file() && m.len() == 512) {
        Ok(())
    } else {
        Err("Add your 512-byte lynxboot.img to Slot/BIOS, then try again.")
    }
}
