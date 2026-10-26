//! Read-only setup checks. Never load a core, unpack a game, or rescan the library.
use slot_store::{Cart, Core, Platform};
use slot_ui::{SetupEntry, SetupStatus as Status};
use std::{fs::File, io::Read, path::Path};

pub fn scan(root: &Path, carts: &[Cart]) -> Vec<SetupEntry> {
    let selected = slot_store::read_selected_cores(root);
    slot_ui::SYSTEM_ORDER.iter().map(|&p| {
        let mut e = SetupEntry { name: p.name().into(), status: Status::Ready, details: Vec::new() };
        if p == Platform::Ports {
            if let Err(message) = crate::ports::check_runtime(root) { missing(&mut e, message); }
            else { e.details.push("PortMaster and native runtime files found.".into()); }
            e.details.push("Install Balatro in PortMaster, then copy your Balatro.exe into Data/ports/balatro. Use the game menu to save and quit.".into());
        } else if p == Platform::Pico8 {
            match crate::pico8::check_runtime(root) {
                Ok(_) => e.details.push("PICO-8 runtime files found.".into()),
                Err(message) => problem(&mut e, nonempty(&root.join("BIOS/PICO8/pico8_64")) && nonempty(&root.join("BIOS/PICO8/pico8.dat")), message)
            }
            for file in ["System/pico8-sdl-nosensor.so", "System/pico8-bin/wget", "System/pico8-controllers.txt", "System/pico8-splore.png"] {
                if !nonempty(&root.join(file)) { missing(&mut e, format!("Restore Slot/{file} from the latest Slot card package.")); }
            }
            e.details.push("Use the Raspberry Pi ARM64 download from your PICO-8 account. Splore needs working Wi-Fi to download carts.".into());
        } else {
            let mut cores = vec![Core::default_for(p)];
            for cart in carts.iter().filter(|c| c.platform == p) {
                if let Some(&core) = selected.get(&cart.stem).filter(|c| c.runs(p)) {
                    if !cores.contains(&core) { cores.push(core); }
                }
            }
            for core in cores {
                let file = format!("System/{}", crate::core::dylib_name(core));
                match elf(&root.join(&file)) {
                    Ok(()) => e.details.push(format!("{}: found.", core.text())),
                    Err(reason) => problem(&mut e, root.join(&file).exists(), format!("{reason}: Slot/{file}. Restore this file from the latest Slot card package.")),
                }
            }
            match p {
                Platform::Lynx => {
                    if let Err(message) = crate::root::check_lynx_bios(root) {
                        problem(&mut e, root.join("BIOS/lynxboot.img").exists(), message.into());
                    } else { e.details.push("Lynx boot ROM: found (lynxboot.img).".into()); }
                    e.details.push("SELECT rotates the screen and controls. L1 / R1 are Option 1 / 2; START is Pause.".into());
                }
                Platform::Ws => {
                    e.details.push("No external BIOS required. Put both .ws and .wsc games in Slot/Games/WS. SELECT rotates the screen and controls.".into());
                    e.details.push("Landscape: D-pad = X cursor; R2/R1/L2/L1 = Y up/right/down/left. Portrait: D-pad = Y cursor; face buttons = X cursor; L1/R1 = A/B.".into());
                }
                Platform::SegaCd => sega_cd(root, &mut e),
                Platform::PceCd => {
                    if let Err(message) = crate::pce::check_bios(root) { problem(&mut e, root.join("BIOS/syscard3.pce").exists(), message.into()); }
                    else { e.details.push("System Card 3: found (syscard3.pce).".into()); }
                }
                Platform::NeoGeo => neogeo(root, &mut e),
                Platform::Ps1 => ps1(root, &mut e),
                Platform::Gba => {
                    if crate::root::has_real_bios(root) { e.details.push("Optional gba_bios.bin: found.".into()); }
                    else { e.details.push("GBA BIOS is optional. For the boot screen, add gba_bios.bin (16 KB) to Slot/BIOS. Games can use the built-in BIOS.".into()); }
                }
                _ => e.details.push("No external BIOS is required.".into()),
            }
        }
        e.details.push(format!("Games: Slot/Games/{}/", p.dir_name()));
        e.details.push(format!("Formats: {}", p.extensions().join(", ")));
        if p == Platform::NeoGeo { e.details.push("Use FBNeo ROM sets. Keep original ZIP names and leave archives intact. Split clones also need their parent ZIP.".into()); }
        if p.is_disc() { e.details.push("For CUE games, keep every referenced track beside its CUE file. Use M3U for multiple discs.".into()); }
        e.details.push("Checks cover setup files, not every game's compatibility. If a launch fails, see Slot/Logs/slot.log.".into());
        e
    }).collect()
}
fn missing(e: &mut SetupEntry, text: String) {
    problem(e, false, text);
}
fn problem(e: &mut SetupEntry, exists: bool, text: String) {
    if !exists {
        e.status = Status::Missing;
    } else if e.status != Status::Missing {
        e.status = Status::Check;
    }
    e.details.push(text);
}
fn advisory(e: &mut SetupEntry, status: Status) {
    if e.status == Status::Ready {
        e.status = status;
    }
}
fn nonempty(path: &Path) -> bool {
    path.metadata().is_ok_and(|m| m.is_file() && m.len() > 0)
}
fn sized(path: &Path, size: u64) -> bool {
    path.metadata()
        .is_ok_and(|m| m.is_file() && m.len() == size)
}
fn elf(path: &Path) -> Result<(), &'static str> {
    let mut h = [0; 64];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut h))
        .map_err(|_| "Missing or unreadable emulator")?;
    let machine = if cfg!(target_arch = "aarch64") {
        183
    } else if cfg!(target_arch = "x86_64") {
        62
    } else {
        0
    };
    if &h[..4] != b"\x7fELF"
        || h[4] != 2
        || h[5] != 1
        || u16::from_le_bytes([h[16], h[17]]) != 3
        || u16::from_le_bytes([h[18], h[19]]) != machine
    {
        return Err("Wrong emulator build");
    }
    Ok(())
}
fn sega_cd(root: &Path, e: &mut SetupEntry) {
    let mut found = 0;
    for (region, names) in ["USA", "Europe", "Japan"]
        .into_iter()
        .zip(crate::sega_cd::BIOS_NAMES.chunks(4))
    {
        let present = names.iter().any(|name| {
            let bin = root.join(format!("BIOS/{name}.bin"));
            let zip = root.join(format!("BIOS/{name}.zip"));
            sized(&bin, 128 * 1024)
                || zip_entries(&zip).is_some_and(|entries| {
                    entries.iter().any(|x| {
                        !x.encrypted() && x.size == 128 * 1024 && matches!(x.method, 0 | 8)
                    })
                })
        });
        if present {
            found += 1;
            e.details.push(format!("{region} BIOS: found."));
        } else {
            e.details.push(format!(
                "{region}: add {}.bin (128 KB) to Slot/BIOS if you play this region.",
                names[0]
            ));
        }
    }
    if found == 0 {
        e.status = Status::Missing;
    } else if found < 3 {
        advisory(e, Status::Check);
    }
}
fn zip_entries(path: &Path) -> Option<Vec<slot_store::archive::zip::ZipEntry>> {
    // BIOS archives are small; never allocate a central directory from an arbitrary giant file.
    let mut f = File::open(path).ok()?;
    if f.metadata().ok()?.len() > 32 * 1024 * 1024 {
        return None;
    }
    slot_store::archive::zip::index(&mut f).ok()
}
fn neogeo(root: &Path, e: &mut SetupEntry) {
    let paths = ["BIOS/neogeo.zip", "Games/NEOGEO/neogeo.zip"];
    let valid = paths.iter().find(|p| {
        zip_entries(&root.join(p)).is_some_and(|chips| {
            let has = |name: &str, size| {
                chips.iter().any(|x| {
                    x.name == name && x.size == size && !x.encrypted() && matches!(x.method, 0 | 8)
                })
            };
            ["sp-s3.sp1", "sp-s2.sp1", "sp-s.sp1"]
                .iter()
                .any(|n| has(n, 0x20000))
                && has("sm1.sm1", 0x20000)
                && has("sfix.sfix", 0x20000)
                && has("000-lo.lo", 0x20000)
        })
    });
    if let Some(path) = valid {
        e.details.push(format!(
            "Slot/{path}: required MVS BIOS chips listed. FBNeo verifies their contents at launch."
        ));
    } else {
        problem(e, paths.iter().any(|p| root.join(p).exists()), "Add a complete FBNeo neogeo.zip to Slot/BIOS. The archive needs a European MVS BIOS, sm1.sm1, sfix.sfix and 000-lo.lo; do not extract it.".into());
    }
}
fn ps1(root: &Path, e: &mut SetupEntry) {
    let names = [
        "scph5500",
        "scph5501",
        "scph5502",
        "psxonpsp660",
        "scph101",
        "scph7001",
        "scph1001",
    ];
    let found: Vec<_> = names
        .iter()
        .filter(|n| sized(&root.join(format!("BIOS/{n}.bin")), 512 * 1024))
        .collect();
    if found.is_empty() {
        advisory(e, Status::Optional);
        e.details.push("No standard PlayStation BIOS found. Built-in BIOS emulation is available, but a real BIOS improves compatibility.".into());
        e.details.push("Add scph5500.bin (Japan), scph5501.bin (USA), or scph5502.bin (Europe), 512 KB each, to Slot/BIOS.".into());
        e.details
            .push("Other BIOS names may also be recognized by the emulator at launch.".into());
    } else {
        e.details.push(format!(
            "PlayStation BIOS files: {}.",
            found
                .iter()
                .map(|n| format!("{n}.bin"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
}
