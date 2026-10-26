//! Sega CD setup. The actual regional choice is made by PicoDrive from the disc.
use std::path::Path;

pub const BIOS_NAMES: [&str; 12] = [
    "bios_CD_U", "us_scd2_9306", "SegaCDBIOS9303", "us_scd1_9210",
    "bios_CD_E", "eu_mcd2_9306", "eu_mcd2_9303", "eu_mcd1_9210",
    "bios_CD_J", "jp_mcd2_921222", "jp_mcd1_9112", "jp_mcd1_9111",
];

pub fn check_bios(root: &Path) -> Result<(), &'static str> {
    let dir = root.join("BIOS");
    if BIOS_NAMES.iter().any(|name| {
        dir.join(format!("{name}.bin")).metadata().is_ok_and(|m| m.is_file() && m.len() == 128 * 1024)
            || dir.join(format!("{name}.zip")).metadata().is_ok_and(|m| m.is_file() && m.len() > 0)
    }) {
        Ok(())
    } else {
        Err("Add your 128 KB region BIOS to Slot/BIOS: bios_CD_U.bin, bios_CD_E.bin or bios_CD_J.bin.")
    }
}
