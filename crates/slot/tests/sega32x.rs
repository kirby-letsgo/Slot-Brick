mod common;

use slot::app::{App, Phase};
use slot_input::Btn;
use slot_store::{Core, Platform, Shader};

#[test]
fn scan_and_search_keep_32x_separate_from_mega_drive() {
    let d = tempfile::tempdir().unwrap();
    slot::root::ensure(d.path());
    for dir in ["Games", "Labels", "States", "Saves", "Overlays"] {
        assert!(d.path().join(dir).join("32X").is_dir());
    }
    std::fs::write(d.path().join("Games/32X/Knuckles.32X"), [0; 512]).unwrap();
    std::fs::write(d.path().join("Games/MD/Knuckles.md"), [0; 512]).unwrap();
    std::fs::write(d.path().join("Labels/32X/Knuckles.png"), b"label").unwrap();
    let carts = slot_store::scan(d.path()).unwrap();
    let found = carts.iter().find(|c| c.platform == Platform::S32x).unwrap();
    assert!(found.label.is_some());
    assert_eq!(Core::default_for(found.platform), Core::PicoDrive);
    assert!(!Core::GenesisPlusGx.runs(found.platform));
    assert!(!Core::PicoDrive.runs(Platform::Md));
    let mut app = App::new(carts);
    app.show_systems();
    assert!(app.system_grid().unwrap().entries.contains(&(Platform::S32x, 1)));
    app.search_games("Knuckles");
    assert_eq!(app.game_search_text(), Some(("Knuckles", 2)));
    assert_ne!(Platform::S32x.game_key("Knuckles"), Platform::Md.game_key("Knuckles"));
}

#[test]
fn settings_and_six_button_controls_are_independent() {
    let d = tempfile::tempdir().unwrap();
    slot::root::ensure(d.path());
    let mut state = slot_store::SlotState::default();
    state.shader_32x = Shader::Grid;
    state.set_overlay(Platform::S32x, Some("32X Frame.png".into()));
    slot_store::write_slot_state(d.path(), &state).unwrap();
    let state = slot_store::read_slot_state(d.path());
    assert_eq!(state.shader_32x, Shader::Grid);
    assert_ne!(state.shader_md, Shader::Grid);
    assert_eq!(state.overlay(Platform::S32x), Some("32X Frame.png"));
    assert_eq!(state.overlay(Platform::Md), None);

    std::fs::write(d.path().join("Games/32X/Test.32x"), [0; 512]).unwrap();
    let session = common::session_playing(d.path());
    let app = session.app();
    assert!(matches!(app.phase(), Phase::Playing { .. }));
    assert_eq!(app.core(), Core::PicoDrive);
    assert!(!app.taken_buttons().contains(&Btn::L1));
    assert!(!app.taken_buttons().contains(&Btn::R1));
    assert_eq!(app.canvas(), (320, 240));
}

#[test]
fn shader_menu_updates_only_32x() {
    let d = common::tmp_root_with_carts(&["One", "Two"]);
    common::clocked(d.path());
    let (mut app, _) = common::app_booting_at(d.path(), common::CLOCK_IS_SET);
    common::open_at(&mut app, slot_ui::QuickRow::S32xShader);
    common::press(&mut app, Btn::Right);
    let state = slot_store::read_slot_state(d.path());
    assert_eq!(state.shader_32x, Shader::Grid);
    assert_eq!(state.shader_md, Shader::Off);
    common::press(&mut app, Btn::Down);
    assert_eq!(app.quick_menu(), Some(slot_ui::QuickRow::S32xOverlay));
}

#[test]
#[ignore = "requires native PicoDrive and a homebrew 32X ROM"]
fn native_32x_renders_and_restores_states_with_the_dynarec() {
    use slot_retro::ButtonMask;
    let d = tempfile::tempdir().unwrap();
    slot::root::ensure(d.path());
    let core = std::env::var_os("SLOT_TEST_32X_CORE").expect("native PicoDrive path");
    std::fs::copy(core, d.path().join("System/picodrive_libretro.so")).unwrap();
    let rom = std::path::PathBuf::from(std::env::var_os("SLOT_TEST_32X_ROM").expect("homebrew ROM"));
    for _ in 0..2 {
        let opened = slot::core::open_core(d.path(), Core::PicoDrive, "", false, None, None);
        assert!(opened.named);
        let mut core = opened.core;
        core.load(&rom).unwrap();
        assert_eq!(core.canvas(), (320, 240));
        assert!((50.0..61.0).contains(&core.av_info().fps));
        for _ in 0..180 { core.run_frame(ButtonMask::default()); core.take_audio(); }
        let pixels = core.video_xrgb8888();
        assert!(pixels.chunks_exact(4).any(|p| p[..3] != pixels[..3]), "homebrew draws a non-uniform 32X frame");
        let state = core.serialize().unwrap();
        assert!(state.len() > 256 * 1024, "32X RAM is included");
        core.run_frame(ButtonMask(ButtonMask::START));
        core.unserialize(&state).unwrap();
        for _ in 0..10 { core.run_frame(ButtonMask::default()); core.take_audio(); }
    }
}
