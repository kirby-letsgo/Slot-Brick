//! The CD System Card is supplied by the player, not bundled with Slot.
use std::path::Path;

pub fn check_bios(root: &Path) -> Result<(), &'static str> {
    if root.join("BIOS/syscard3.pce").metadata().is_ok_and(|m|
        m.is_file() && matches!(m.len(), 262144 | 262656)) {
        Ok(())
    } else {
        Err("Add your System Card 3 BIOS (syscard3.pce, 256 KB) to Slot/BIOS.")
    }
}
