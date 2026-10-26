//! PortMaster keeps scripts and game data together at the card root, outside Slot.
use std::path::{Path, PathBuf};
use crate::{Cart, Platform};

pub fn directory(root: &Path) -> PathBuf {
    root.parent().unwrap_or(root).join("Data/ports")
}

pub fn manager(root: &Path) -> PathBuf {
    root.parent().unwrap_or(root).join("Apps/PortMaster/PortMaster")
}

pub fn scan(root: &Path) -> Vec<Cart> {
    let mut carts = Vec::new();
    if let Ok(entries) = std::fs::read_dir(directory(root)) {
        for entry in entries.flatten() {
            let rom = entry.path();
            let Some(stem) = rom.file_stem().and_then(|s| s.to_str()) else { continue; };
            if stem.starts_with('.') || !Platform::Ports.accepts(&rom) || !rom.is_file() { continue; }
            let label = crate::labels_of(root, Platform::Ports, stem).0.or_else(|| {
                let preview = root.parent()?.join("Imgs/PORTS").join(format!("{stem}-pre.png"));
                preview.is_file().then_some(preview)
            });
            carts.push(Cart { platform: Platform::Ports, stem: stem.into(), rom: rom.clone(),
                label, famicom_label: None, title: String::new(), code: String::new(), shell: None });
        }
    }
    if root.join("System/portmaster-launch.sh").is_file() {
        let mut stem = "PortMaster".to_string();
        while carts.iter().any(|c| c.stem == stem) { stem.push_str(" (Manager)"); }
        carts.push(Cart { platform: Platform::Ports, stem, rom: root.join("System/portmaster-launch.sh"),
            label: manager(root).join("screenshot.png").is_file().then(|| manager(root).join("screenshot.png")),
            famicom_label: None, title: String::new(), code: "SLOT_PORTMASTER".into(), shell: None });
    }
    carts.sort_by_key(|c| (c.code != "SLOT_PORTMASTER", c.stem.to_lowercase()));
    carts
}
