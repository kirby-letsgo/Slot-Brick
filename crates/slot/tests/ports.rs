mod common;
use std::{fs, path::Path};
use slot::{app::{App, Phase}, ports};
use slot_store::{Cart, Platform};

fn card() -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("Slot");
    slot::root::ensure(&root);
    fs::create_dir_all(t.path().join("Data/ports/balatro")).unwrap();
    fs::write(t.path().join("Data/ports/Balatro.sh"), "#!/bin/bash\n").unwrap();
    fs::write(root.join("System/portmaster-launch.sh"), "#!/bin/bash\n").unwrap();
    for file in ["control.txt", "pugwash", "gamecontrollerdb.txt", "runtimes/love_11.5/love.aarch64"] {
        let p = slot_store::ports::manager(&root).join(file);
        fs::create_dir_all(p.parent().unwrap()).unwrap(); fs::write(p, "fixture").unwrap();
    }
    fs::create_dir_all(root.join("System/PortRuntime/bin")).unwrap();
    for file in ["bash", "python3"] { fs::write(root.join("System/PortRuntime/bin").join(file), "fixture").unwrap(); }
    t
}
fn game(root: &Path) -> Cart { slot_store::ports::scan(root).into_iter().find(|c| c.stem=="Balatro").unwrap() }

#[test]
fn scans_only_top_level_launchers_and_refreshes_ports_even_from_a_cached_library() {
    let t = card(); let root = t.path().join("Slot");
    fs::write(t.path().join("Data/ports/.hidden.sh"), "").unwrap();
    fs::write(t.path().join("Data/ports/balatro/helper.sh"), "").unwrap();
    let carts = slot_store::scan(&root).unwrap();
    assert_eq!(carts.len(), 2);
    assert!(carts.iter().all(|c| c.platform==Platform::Ports));
    slot_store::scan_cache::write(&root, &carts).unwrap();
    fs::write(t.path().join("Data/ports/Other.sh"), "").unwrap();
    let cached = slot_store::scan_cache::read(&root).unwrap();
    assert_eq!(cached.len(), 3);
    fs::remove_file(t.path().join("Data/ports/Balatro.sh")).unwrap();
    assert_eq!(slot_store::scan_cache::read(&root).unwrap().len(), 2);
}
#[test]
fn balatro_has_actionable_missing_data_guidance_and_accepts_the_windows_game_file() {
    let t=card(); let root=t.path().join("Slot");
    let error=ports::prepare(&root,&game(&root)).unwrap_err();
    assert!(error.contains("Balatro.exe")); assert!(error.contains("Data/ports/balatro"));
    fs::write(t.path().join("Data/ports/balatro/Balatro.exe"), "purchased data fixture").unwrap();
    let launch=ports::prepare(&root,&game(&root)).unwrap();
    ports::queue(&root,&launch).unwrap();
    assert_eq!(fs::read_to_string(root.join("Config/port-launch")).unwrap(),launch.script.unwrap().to_str().unwrap());
}
#[test]
fn manager_is_distinct_from_a_same_named_game_and_never_requests_a_rom() {
    let t=card(); let root=t.path().join("Slot");
    fs::write(t.path().join("Data/ports/PortMaster.sh"), "").unwrap();
    let carts=slot_store::ports::scan(&root);
    let manager=&carts[0];
    assert_eq!(manager.code,"SLOT_PORTMASTER");
    assert_ne!(manager.stem,"PortMaster");
    let launch=ports::prepare(&root,manager).unwrap();
    assert!(launch.script.is_none()); ports::queue(&root,&launch).unwrap();
    assert_eq!(fs::read(root.join("Config/port-launch")).unwrap(),b"");
}
#[test]
fn scripts_cannot_escape_the_ports_directory_through_symlinks() {
    use std::os::unix::fs::symlink;
    let t=card(); let root=t.path().join("Slot");
    let outside=t.path().join("outside.sh");fs::write(&outside,"exit 0").unwrap();
    symlink(&outside,t.path().join("Data/ports/Outside.sh")).unwrap();
    let launch=ports::Launch{script:Some(t.path().join("Data/ports/Outside.sh"))};
    assert!(ports::queue(&root,&launch).is_err());
    assert!(!ports::pending(&root));
}
#[test]
fn returning_from_native_ports_keeps_the_shelf_without_an_autolaunch_loop() {
    let t=card();let root=t.path().join("Slot"); common::clocked(&root);
    let mut state=slot_store::read_slot_state(&root);
    state.cart=Some("Balatro".into());state.cart_platform=Some(Platform::Ports);
    slot_store::write_slot_state(&root,&state).unwrap();
    let app=App::boot(&root);
    assert!(matches!(app.phase(),Phase::Shelf));
    assert_eq!(app.selected_cart().unwrap().stem,"Balatro");
}
#[test]
fn ports_search_and_setup_use_native_support_not_a_libretro_core() {
    let t=card();let root=t.path().join("Slot");
    let carts=slot_store::scan(&root).unwrap();
    let setup=slot::system_setup::scan(&root,&carts);
    let ports=setup.iter().find(|e|e.name=="Ports").unwrap();
    assert!(!ports.details.iter().any(|d|d.contains("libretro")));
    let mut app=App::new(carts);app.search_games("Balatro");
    assert_eq!(app.game_search_text(),Some(("Balatro",1)));
    assert_eq!(app.selected_cart().unwrap().platform,Platform::Ports);
}

#[test]
fn native_runner_passes_paths_as_arguments_and_reports_nonzero_exit() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let t=card();let root=t.path().join("Slot");
    let bash=root.join("System/PortRuntime/bin/bash");fs::remove_file(&bash).unwrap();symlink("/bin/sh",&bash).unwrap();
    fs::write(root.join("System/portmaster-launch.sh"),"#!/bin/sh\nprintf '%s' \"$1\" > \"$SLOT_ROOT/Logs/launched\"\nexit 7\n").unwrap();
    fs::set_permissions(root.join("System/portmaster-launch.sh"),fs::Permissions::from_mode(0o755)).unwrap();
    let script=slot_store::ports::directory(&root).join("Game $(touch WRONG); quoted.sh");
    fs::write(&script,"unused fixture").unwrap();
    ports::queue(&root,&ports::Launch{script:Some(script.clone())}).unwrap();
    let result=ports::run_pending(&root).unwrap_err();
    assert!(result.contains("7"));assert!(result.contains("ports.log"));
    assert_eq!(fs::read_to_string(root.join("Logs/launched")).unwrap(),script.to_str().unwrap());
    assert!(!ports::pending(&root));
    assert!(!root.join("WRONG").exists());
}
