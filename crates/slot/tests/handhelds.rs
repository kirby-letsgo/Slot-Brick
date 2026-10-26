mod common;
use slot::app::Phase;
use slot_input::{Action, Btn};
use slot_store::{Core, Platform, Shader};
use std::{fs, path::PathBuf};

#[test]
fn mono_and_color_share_one_shelf_and_search_while_lynx_saves_stay_separate() {
    let d = common::tmp_root_with_carts(&[]);
    for path in ["WS/Mono.ws", "WS/Color.WSC", "LYNX/Mono.lnx"] {
        fs::write(d.path().join("Games").join(path), [0; 128]).unwrap();
    }
    let carts = slot_store::scan(d.path()).unwrap();
    assert_eq!(carts.iter().filter(|c| c.platform == Platform::Ws).count(), 2);
    let mut app = slot::app::App::new(carts);
    app.search_games("Mono");
    assert_eq!(app.game_search_text(), Some(("Mono", 2)));
    for (platform, bytes) in [(Platform::Ws, b"ws save"), (Platform::Lynx, b"lx save")] {
        slot::persist::write_sav(d.path(), platform, "Mono", bytes).unwrap();
        assert_eq!(slot::persist::read_sav(d.path(), platform, "Mono").unwrap(), bytes);
    }
    for (p, c) in [(Platform::Ws, Core::BeetleWswan), (Platform::Lynx, Core::BeetleLynx)] {
        assert_eq!(Core::default_for(p), c);
        assert!(p.takes_archives());
        assert!(!p.reads_bare_keys());
        for dir in ["Games", "Labels", "Saves", "States", "Overlays"] { assert!(d.path().join(dir).join(p.dir_name()).is_dir()); }
    }
}

#[test]
fn lynx_requires_a_complete_bios_and_returns_to_the_shelf_with_guidance() {
    let d = common::tmp_root_with_carts(&[]);
    fs::write(d.path().join("Games/LYNX/Game.lnx"), [0; 128]).unwrap();
    common::clocked(d.path());
    let mut session = slot::session::Session::boot(d.path().into());
    common::open_shelf(session.app_mut());
    session.app_mut().apply(Action::Insert);
    for _ in 0..120 { session.update(1.0 / 60.0); }
    assert!(matches!(session.app().phase(), Phase::Shelf));
    assert!(!session.has_core());
    for size in [0, 511, 513] {
        fs::write(d.path().join("BIOS/lynxboot.img"), vec![0; size]).unwrap();
        assert!(slot::root::check_lynx_bios(d.path()).is_err());
    }
    fs::write(d.path().join("BIOS/lynxboot.img"), vec![0; 512]).unwrap();
    assert!(slot::root::check_lynx_bios(d.path()).is_ok());
}

#[test]
fn missing_handheld_emulator_returns_to_the_shelf_instead_of_running_a_mock_frame() {
    let d = common::tmp_root_with_carts(&[]);
    fs::write(d.path().join("Games/WS/Game.ws"), [0; 128]).unwrap();
    common::clocked(d.path());
    let mut session = slot::session::Session::boot(d.path().into());
    common::open_shelf(session.app_mut());
    session.app_mut().apply(Action::Insert);
    for _ in 0..120 { session.update(1.0 / 60.0); }
    assert!(matches!(session.app().phase(), Phase::Shelf));
    assert!(!session.has_core());
}

#[test]
fn screen_settings_persist_independently_and_shoulders_reach_the_handhelds() {
    let d = common::tmp_root_with_carts(&[]); common::clocked(d.path());
    let mut app = slot::app::App::boot(d.path());
    common::open_at(&mut app, slot_ui::QuickRow::WsShader);
    common::press(&mut app, Btn::Right);
    let state = slot_store::read_slot_state(d.path());
    assert_eq!(state.shader_ws, Shader::Grid);
    assert_eq!(state.shader_lynx, Shader::Off);
    assert_eq!(state.shader_ngp, Shader::Off);
    for p in [Platform::Ws, Platform::Lynx] {
        app.set_platform(p);
        assert!(app.taken_buttons().is_empty());
    }
}

#[test]
#[ignore = "requires native handheld cores and make-handhelds.py diagnostic programs"]
fn native_cores_render_rotate_restore_and_keep_battery_ram() {
    use slot_retro::ButtonMask;
    let root = PathBuf::from(std::env::var_os("SLOT_TEST_HANDHELDS_ROOT").unwrap());
    for (which, path, canvas) in [
        (Core::BeetleWswan, "WS/Mono Probe.ws", (448, 336)),
        (Core::BeetleWswan, "WS/Color Probe.wsc", (448, 336)),
        (Core::BeetleLynx, "LYNX/Lynx Probe.o", (320, 240)),
    ] {
        let opened = slot::core::open_core(&root, which, "", false, None, None);
        assert!(opened.named);
        let mut core = opened.core;
        core.load(&root.join("Games").join(path)).unwrap();
        for _ in 0..12 { core.run_frame(ButtonMask::default()); core.take_audio(); }
        assert_eq!(core.canvas(), canvas);
        let before = core.video_xrgb8888().to_vec();
        assert!(before.chunks_exact(4).any(|p| p[..3] != [0, 0, 0]), "visible diagnostic pixels: {path}");
        let state = core.serialize().unwrap();
        assert!(!state.is_empty());
        core.run_frame(ButtonMask(ButtonMask::SELECT));
        core.run_frame(ButtonMask::default());
        assert_ne!(before, core.video_xrgb8888(), "rotation changes the frame: {path}");
        core.unserialize(&state).unwrap();
        if which == Core::BeetleWswan {
            let mut save = core.save_ram().expect("battery SRAM");
            assert_eq!(save.len(), 8192);
            save[0] = 0x5a; core.load_save_ram(&save).unwrap();
            assert_eq!(core.save_ram().unwrap(), save);
        }
    }
}
