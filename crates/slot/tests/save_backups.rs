mod common;

use std::path::{Path, PathBuf};

use common::tmp_root_with_carts;
use slot::persist::{backup_sav, SAV_BACKUPS};
use slot::session::Session;
use slot_input::Action;
use slot_store::Platform;

fn copy(root: &Path, n: usize) -> PathBuf {
    root.join(format!("Saves/Backups/GBA/Emerald ({n}).sav"))
}

fn copies(root: &Path) -> Vec<Vec<u8>> {
    (1..=SAV_BACKUPS + 1)
        .filter_map(|n| std::fs::read(copy(root, n)).ok())
        .collect()
}

#[test]
fn starting_a_game_keeps_a_copy_of_its_save() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    common::clocked(d.path());
    std::fs::write(d.path().join("Saves/GBA/Emerald.sav"), b"before").expect("write sav");
    let mut s = Session::boot(d.path().to_path_buf());
    s.app_mut().apply(Action::Insert);
    for _ in 0..90 {
        s.update(1.0 / 60.0);
    }
    assert!(s.has_core(), "the game never started");
    assert_eq!(copies(d.path()), [b"before"]);
}

#[test]
fn a_game_with_no_save_gets_no_copy() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    common::clocked(d.path());
    let mut s = Session::boot(d.path().to_path_buf());
    s.app_mut().apply(Action::Insert);
    for _ in 0..90 {
        s.update(1.0 / 60.0);
    }
    assert!(s.has_core(), "the game never started");
    assert!(!d.path().join("Saves/Backups").exists());
}

#[test]
fn the_newest_copies_are_kept_and_the_oldest_goes() {
    let d = tmp_root_with_carts(&["Emerald"]);
    for sav in [b"one", b"two", b"two", b"six", b"ten"] {
        backup_sav(d.path(), Platform::Gba, "Emerald", sav).expect("backup");
    }
    assert_eq!(copies(d.path()), [b"ten", b"six", b"two"]);
}
