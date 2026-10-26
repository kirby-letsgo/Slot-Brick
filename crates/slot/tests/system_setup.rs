mod common;
use slot_input::{Action, Btn};
use slot_store::{Core, Platform};
use slot_ui::{QuickRow, SetupStatus as Status};
use std::{fs, path::Path};

fn core(root: &Path, which: Core) {
    let mut h = vec![0; 64];
    h[..4].copy_from_slice(b"\x7fELF");
    h[4] = 2;
    h[5] = 1;
    h[16] = 3;
    h[18..20].copy_from_slice(
        &(if cfg!(target_arch = "aarch64") {
            183u16
        } else {
            62
        })
        .to_le_bytes(),
    );
    fs::write(root.join("System").join(slot::core::dylib_name(which)), h).unwrap();
}
fn entry(root: &Path, p: Platform) -> slot_ui::SetupEntry {
    slot::system_setup::scan(root, &[])
        .into_iter()
        .find(|e| e.name == p.name())
        .unwrap()
}
#[test]
fn lists_every_system_without_reading_games_and_distinguishes_optional_bios() {
    let d = common::tmp_root_with_carts(&[]);
    for c in Core::ALL {
        if c != Core::Pico8 {
            core(d.path(), c);
        }
    }
    // A directory posing as a game cannot be opened as ROM data. Setup only reads setup files.
    fs::create_dir(d.path().join("Games/NEOGEO/mslug.zip")).unwrap();
    let entries = slot::system_setup::scan(d.path(), &[]);
    assert_eq!(entries.len(), Platform::ALL.len());
    assert_eq!(entry(d.path(), Platform::Gba).status, Status::Ready);
    assert_eq!(entry(d.path(), Platform::N64).status, Status::Ready);
    assert_eq!(entry(d.path(), Platform::Ps1).status, Status::Optional);
    assert_eq!(entry(d.path(), Platform::PceCd).status, Status::Missing);
    assert_eq!(entry(d.path(), Platform::NeoGeo).status, Status::Missing);
    assert_eq!(entry(d.path(), Platform::Pico8).status, Status::Missing);
    assert!(!d.path().join("System/Cache/Unpacked").exists());
}
#[test]
fn truncated_and_wrong_architecture_cores_are_not_ready() {
    let d = common::tmp_root_with_carts(&[]);
    let path = d.path().join("System/parallel_n64_libretro.so");
    fs::write(&path, b"\x7fELF").unwrap();
    assert_eq!(entry(d.path(), Platform::N64).status, Status::Check);
    core(d.path(), Core::ParallelN64);
    let mut h = fs::read(&path).unwrap();
    h[18..20].copy_from_slice(&40u16.to_le_bytes());
    fs::write(&path, h).unwrap();
    assert_eq!(entry(d.path(), Platform::N64).status, Status::Check);
    core(d.path(), Core::ParallelN64);
    assert_eq!(entry(d.path(), Platform::N64).status, Status::Ready);
}
#[test]
fn disc_bios_sizes_and_sega_regions_are_independent() {
    let d = common::tmp_root_with_carts(&[]);
    core(d.path(), Core::PicoDrive);
    core(d.path(), Core::BeetlePceFast);
    fs::write(d.path().join("BIOS/syscard3.pce"), [0; 512]).unwrap();
    assert_eq!(entry(d.path(), Platform::PceCd).status, Status::Check);
    fs::write(d.path().join("BIOS/syscard3.pce"), vec![0; 262144]).unwrap();
    assert_eq!(entry(d.path(), Platform::PceCd).status, Status::Ready);
    fs::write(d.path().join("BIOS/bios_CD_U.bin"), vec![0; 128 * 1024]).unwrap();
    let e = entry(d.path(), Platform::SegaCd);
    assert_eq!(e.status, Status::Check);
    assert!(e.details.iter().any(|d| d == "USA BIOS: found."));
    assert!(e.details.iter().any(|d| d.contains("bios_CD_E.bin")));
    for name in ["bios_CD_E", "bios_CD_J"] {
        fs::write(
            d.path().join(format!("BIOS/{name}.bin")),
            vec![0; 128 * 1024],
        )
        .unwrap();
    }
    assert_eq!(entry(d.path(), Platform::SegaCd).status, Status::Ready);
}
#[test]
fn pico8_needs_the_arm64_runtime_data_and_slot_helpers() {
    let d = common::tmp_root_with_carts(&[]);
    let mut h = vec![0; 64];
    h[..4].copy_from_slice(b"\x7fELF");
    h[4] = 2;
    h[5] = 1;
    h[18] = 183;
    fs::write(d.path().join("BIOS/PICO8/pico8_64"), h).unwrap();
    fs::write(d.path().join("BIOS/PICO8/pico8.dat"), "data").unwrap();
    assert_eq!(entry(d.path(), Platform::Pico8).status, Status::Missing);
    fs::create_dir_all(d.path().join("System/pico8-bin")).unwrap();
    for name in [
        "pico8-sdl-nosensor.so",
        "pico8-bin/wget",
        "pico8-controllers.txt",
        "pico8-splore.png",
    ] {
        fs::write(d.path().join("System").join(name), "helper").unwrap();
    }
    assert_eq!(entry(d.path(), Platform::Pico8).status, Status::Ready);
}
#[test]
fn settings_recheck_keeps_selection_and_b_returns_through_details() {
    let d = common::tmp_root_with_carts(&[]);
    common::clocked(d.path());
    let mut app = slot::app::App::boot(d.path());
    common::open_at(&mut app, QuickRow::SystemSetup);
    common::press(&mut app, Btn::A);
    assert_eq!(app.system_setup().unwrap().selected, 0);
    for _ in 0..2 {
        common::press(&mut app, Btn::Down);
    }
    common::press(&mut app, Btn::A);
    assert!(app.system_setup().unwrap().detail);
    core(d.path(), Core::Mgba);
    common::press(&mut app, Btn::X);
    let report = app.system_setup().unwrap();
    assert_eq!(report.selected, 2);
    assert!(report.detail);
    assert_eq!(report.entries[2].status, Status::Ready);
    common::press(&mut app, Btn::B);
    assert!(!app.system_setup().unwrap().detail);
    common::press(&mut app, Btn::B);
    assert_eq!(app.quick_menu(), Some(QuickRow::SystemSetup));
    common::press(&mut app, Btn::A);
    app.apply(Action::QuickMenu);
    assert_eq!(app.quick_menu(), Some(QuickRow::SystemSetup));
}
#[test]
fn an_empty_or_damaged_neo_geo_zip_does_not_count_as_a_bios() {
    let d = common::tmp_root_with_carts(&[]);
    core(d.path(), Core::FbNeo);
    fs::write(d.path().join("BIOS/neogeo.zip"), b"PK\x05\x06").unwrap();
    assert_eq!(entry(d.path(), Platform::NeoGeo).status, Status::Check);
}

#[test]
fn neo_geo_checks_the_required_chips_and_accepts_bios_beside_games() {
    let d = common::tmp_root_with_carts(&[]);
    core(d.path(), Core::FbNeo);
    let path = d.path().join("Games/NEOGEO/neogeo.zip");
    // A valid ZIP containing original zero-filled chip placeholders.
    let complete = include_bytes!("fixtures/setup-bios.zip");
    fs::write(&path, complete).unwrap();
    assert_eq!(entry(d.path(), Platform::NeoGeo).status, Status::Ready);
    // Rename the zoom ROM in both headers, leaving the ZIP structurally valid.
    let mut incomplete = complete.to_vec();
    for at in 0..incomplete.len() - 9 {
        if &incomplete[at..at + 9] == b"000-lo.lo" {
            incomplete[at] = b'x';
        }
    }
    fs::write(&path, incomplete).unwrap();
    assert_eq!(entry(d.path(), Platform::NeoGeo).status, Status::Check);
}
