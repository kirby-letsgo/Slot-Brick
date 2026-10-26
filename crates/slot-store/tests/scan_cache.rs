mod common;
use slot_store::{scan, scan_cache, Cart, Platform, ShellChoice};
use std::{fs, time::Instant};

#[test]
fn snapshot_roundtrips_all_systems_paths_and_metadata_without_opening_roms() {
    let d = tempfile::tempdir().unwrap();
    let carts: Vec<_> = Platform::ALL
        .into_iter()
        .map(|platform| Cart {
            platform,
            stem: "Space\t雪\nQuest".into(),
            rom: d
                .path()
                .join("Games")
                .join(platform.dir_name())
                .join("Space\t雪\nQuest.zip"),
            label: Some(
                d.path()
                    .join("Labels")
                    .join(platform.dir_name())
                    .join("art.png"),
            ),
            famicom_label: None,
            title: "HEADER".into(),
            code: "CODE".into(),
            shell: ShellChoice::parse("rounded 123456 clear"),
        })
        .collect();
    scan_cache::write(d.path(), &carts).unwrap();
    let mut expected = carts;
    for cart in &mut expected {
        cart.title.clear();
        cart.code.clear();
    }
    assert_eq!(scan_cache::read(d.path()), Some(expected));
    // None of the ROM paths even exist: loading the index must not visit them.
    assert!(!d.path().join("Games").exists());
}

#[test]
fn same_named_discs_in_separate_folders_keep_a_usable_index() {
    let d = tempfile::tempdir().unwrap();
    for folder in ["First game", "Second game"] {
        let dir = d.path().join("Games/PS1").join(folder);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Disc 1.chd"), b"disc").unwrap();
    }
    let carts = scan(d.path()).unwrap();
    assert_eq!(carts.len(), 2);
    assert_eq!(carts[0].stem, carts[1].stem);
    scan_cache::write(d.path(), &carts).unwrap();
    assert_eq!(scan_cache::read(d.path()), Some(carts));
}

#[test]
fn moved_card_and_splore_are_supported() {
    let d = tempfile::tempdir().unwrap();
    fs::create_dir_all(d.path().join("Games/PICO8")).unwrap();
    fs::write(d.path().join("Games/PICO8/Cart.p8.png"), b"cart").unwrap();
    let carts = scan(d.path()).unwrap();
    assert_eq!(carts.len(), 2);
    scan_cache::write(d.path(), &carts).unwrap();
    let moved = tempfile::tempdir().unwrap();
    fs::create_dir_all(moved.path().join("System/Cache")).unwrap();
    fs::copy(
        d.path().join(scan_cache::FILE),
        moved.path().join(scan_cache::FILE),
    )
    .unwrap();
    let loaded = scan_cache::read(moved.path()).unwrap();
    assert_eq!(loaded.len(), 2);
    assert!(loaded.iter().all(|c| c.rom.starts_with(moved.path())));
    assert!(loaded.iter().any(|c| c.code == "SLOT_SPLORE"));
}

#[test]
fn corrupt_truncated_obsolete_and_missing_indexes_are_discarded() {
    let d = tempfile::tempdir().unwrap();
    assert!(scan_cache::read(d.path()).is_none());
    scan_cache::write(d.path(), &[]).unwrap();
    assert_eq!(scan_cache::read(d.path()), Some(vec![]));
    let file = d.path().join(scan_cache::FILE);
    let good = fs::read(&file).unwrap();
    for n in 0..good.len() {
        fs::write(&file, &good[..n]).unwrap();
        assert!(scan_cache::read(d.path()).is_none());
    }
    let mut bad = good.clone();
    bad[0] ^= 1;
    fs::write(&file, bad).unwrap();
    assert!(scan_cache::read(d.path()).is_none());
}

#[test]
fn fresh_scan_detects_added_removed_and_replaced_archives() {
    let d = tempfile::tempdir().unwrap();
    fs::create_dir_all(d.path().join("Games/GBA")).unwrap();
    let old = d.path().join("Games/GBA/Old.zip");
    let changed = d.path().join("Games/GBA/Changed.zip");
    let zipped = common::zip_of(&[("game.gba", b"rom")], true);
    fs::write(&old, &zipped).unwrap();
    fs::write(&changed, &zipped).unwrap();
    scan_cache::write(d.path(), &scan(d.path()).unwrap()).unwrap();
    fs::remove_file(old).unwrap();
    fs::write(changed, b"broken archive").unwrap();
    fs::write(d.path().join("Games/GBA/New.zip"), zipped).unwrap();
    assert_eq!(scan_cache::read(d.path()).unwrap().len(), 2);
    let fresh = scan(d.path()).unwrap();
    assert_eq!(fresh.len(), 1);
    assert_eq!(fresh[0].stem, "New");
    scan_cache::write(d.path(), &fresh).unwrap();
    assert_eq!(scan_cache::read(d.path()), Some(fresh));
}

#[test]
fn twenty_thousand_games_load_from_one_index_without_rom_access() {
    let d = tempfile::tempdir().unwrap();
    let carts: Vec<_> = (0..20_000)
        .map(|i| Cart {
            platform: Platform::Gba,
            stem: format!("Game {i:05}"),
            rom: d.path().join(format!("Games/GBA/Game {i:05}.zip")),
            label: None,
            famicom_label: None,
            title: String::new(),
            code: String::new(),
            shell: None,
        })
        .collect();
    scan_cache::write(d.path(), &carts).unwrap();
    let began = Instant::now();
    assert_eq!(scan_cache::read(d.path()), Some(carts));
    eprintln!("20,000 games loaded from index in {:?}", began.elapsed());
}
