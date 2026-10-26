mod common;
use slot::app::Phase;
use slot_input::{Action, Btn};
use slot_store::{Core, Platform, Shader};

#[test]
fn scan_groups_discs_tracks_and_playlists_without_other_console_formats() {
    let d = tempfile::tempdir().unwrap();
    slot::root::ensure(d.path());
    let games = d.path().join("Games/PCECD");
    std::fs::create_dir(games.join("Two Discs")).unwrap();
    for (name, text) in [
        ("Single.cue", "FILE \"Single.bin\" BINARY\n"),
        ("Single.bin", "track"),
        ("Compressed.chd", "disc"),
        ("Two Discs/Game.m3u", "Disc 1.cue\nDisc 2.chd\n"),
        ("Two Discs/Disc 1.cue", "FILE \"Track.bin\" BINARY\n"),
        ("Two Discs/Track.bin", "track"),
        ("Two Discs/Disc 2.chd", "disc"),
        ("Wrong.exe", "PlayStation executable"),
        ("Wrong.pbp", "PlayStation container"),
        ("Loose.bin", "track without a cue"),
        ("Archive.zip", "not supported"),
    ] { std::fs::write(games.join(name), text).unwrap(); }
    let carts = slot_store::scan(d.path()).unwrap();
    let names: Vec<_> = carts.iter().map(|c| c.stem.as_str()).collect();
    assert_eq!(names, ["Compressed", "Game", "Single"]);
    assert!(carts.iter().all(|c| c.platform == Platform::PceCd));
    assert_eq!(Core::default_for(Platform::PceCd), Core::BeetlePceFast);
    assert!(Core::BeetlePceFast.runs(Platform::PceCd));
    assert!(!Platform::PceCd.takes_archives());
}

#[test]
fn missing_bios_returns_to_shelf_without_starting_a_core() {
    let d = common::tmp_root_with_carts(&[]);
    std::fs::write(d.path().join("Games/PCECD/Game.chd"), "disc").unwrap();
    assert!(slot::pce::check_bios(d.path()).unwrap_err().contains("syscard3.pce"));
    common::clocked(d.path());
    let mut session = slot::session::Session::boot(d.path().into());
    common::open_shelf(session.app_mut());
    session.app_mut().apply(Action::Insert);
    for _ in 0..120 { session.update(1.0 / 60.0); }
    assert!(matches!(session.app().phase(), Phase::Shelf));
    assert!(!session.has_core());
    let bios = d.path().join("BIOS/syscard3.pce");
    std::fs::write(&bios, [0; 1024]).unwrap();
    assert!(slot::pce::check_bios(d.path()).is_err());
    std::fs::write(&bios, vec![0; 256 * 1024 + 512]).unwrap();
    assert!(slot::pce::check_bios(d.path()).is_ok());
    std::fs::write(&bios, vec![0; 256 * 1024]).unwrap();
    assert!(slot::pce::check_bios(d.path()).is_ok());
}

#[test]
fn disc_selection_labels_and_settings_are_separate_from_playstation() {
    let d = common::tmp_root_with_carts(&[]);
    let games = d.path().join("Games/PCECD");
    std::fs::write(games.join("Other.chd"), "disc").unwrap();
    std::fs::write(games.join("Game.m3u"), "Disc 1.chd\nDisc 2.chd\n").unwrap();
    for n in [1, 2] { std::fs::write(games.join(format!("Disc {n}.chd")), "disc").unwrap(); }
    std::fs::write(d.path().join("Labels/PCECD/Disc 2.png"), "art").unwrap();
    slot_store::ini::write(d.path(), slot::disc_picker::DISCS_FILE, "Game", "2").unwrap();
    common::clocked(d.path());
    let mut app = slot::app::App::boot(d.path());
    common::open_shelf(&mut app);
    app.apply(Action::Insert);
    let faces = app.disc_faces_wanted().unwrap();
    assert_eq!(faces.len(), 2);
    assert!(faces[1].0.label.as_ref().unwrap().ends_with("Labels/PCECD/Disc 2.png"));
    common::press(&mut app, Btn::A);
    assert_eq!(app.chosen_disc(), Some(0), "does not inherit PS1's disc choice");
    assert_eq!(slot_store::ini::value(d.path(), slot::disc_picker::DISCS_FILE, "PCECD/Game").as_deref(), Some("1"));
    assert_eq!(slot_store::ini::value(d.path(), slot::disc_picker::DISCS_FILE, "Game").as_deref(), Some("2"));

    let d = common::tmp_root_with_carts(&["One", "Two"]);
    common::clocked(d.path());
    let mut app = slot::app::App::boot(d.path());
    common::open_at(&mut app, slot_ui::QuickRow::PceCdShader);
    common::press(&mut app, Btn::Right);
    let state = slot_store::read_slot_state(d.path());
    assert_eq!(state.shader_pcecd, Shader::Grid);
    assert_eq!(state.shader_32x, Shader::Off);
    assert_eq!(state.shader_ps1, Shader::Off);
}


#[test]
fn hucards_and_cds_are_separate_search_results_and_saves() {
    let d = common::tmp_root_with_carts(&[]);
    for (dir, ext) in [("PCE", "pce"), ("PCECD", "chd")] {
        std::fs::write(d.path().join(format!("Games/{dir}/Same.{ext}")), [0; 8192]).unwrap();
    }
    let carts = slot_store::scan(d.path()).unwrap();
    assert_eq!(carts.len(), 2);
    assert!(Platform::Pce.takes_archives());
    assert!(!Platform::Pce.accepts(std::path::Path::new("CD.cue")));
    assert_eq!(Core::default_for(Platform::Pce), Core::BeetlePceFast);
    common::clocked(d.path());
    let mut app = slot::app::App::boot(d.path());
    app.search_games("Same");
    assert_eq!(app.game_search_text(), Some(("Same", 2)));
    common::press(&mut app, Btn::B);
    common::open_at(&mut app, slot_ui::QuickRow::PceShader);
    common::press(&mut app, Btn::Right);
    let state = slot_store::read_slot_state(d.path());
    assert_eq!(state.shader_pce, Shader::Grid);
    assert_eq!(state.shader_pcecd, Shader::Off);
    for dir in ["PCE", "PCECD"] {
        assert!(d.path().join(format!("Saves/{dir}")).is_dir());
        assert!(d.path().join(format!("States/{dir}")).is_dir());
    }
}

#[test]
#[ignore = "requires native Beetle PCE Fast and generated diagnostic card"]
fn native_hucard_and_cd_formats_keep_bram_states_and_disc_selection() {
    use slot_retro::ButtonMask;
    let root = std::path::PathBuf::from(std::env::var_os("SLOT_TEST_PCE_ROOT").unwrap());
    for (platform, rom) in [
        (Platform::Pce, "Games/PCE/HuCard Test.pce"),
        (Platform::PceCd, "Games/PCECD/CD Test (Disc 1).cue"),
        (Platform::PceCd, "Games/PCECD/CD Test.chd"),
        (Platform::PceCd, "Games/PCECD/CD Test.m3u"),
    ] {
        let opened = slot::core::open_pce(&root, platform);
        assert!(opened.named);
        let mut core = opened.core;
        let playlist = rom.ends_with("m3u");
        if playlist { core.want_disc(1); }
        core.load(&root.join(rom)).unwrap();
        core.settle_disc();
        if playlist { assert_eq!(core.discs(), Some((1, 2))); }
        for _ in 0..120 { core.run_frame(ButtonMask::default()); core.take_audio(); }
        assert_eq!(core.canvas(), (320, 240));
        assert!(core.video_xrgb8888().chunks_exact(4).any(|p| p[1] > 100 && p[0] < 50), "diagnostic draws green");
        let mut bram = core.save_ram().unwrap();
        assert_eq!(bram.len(), 2048);
        bram[123] = 0x5a;
        core.load_save_ram(&bram).unwrap();
        let state = core.serialize().unwrap();
        core.run_frame(ButtonMask::default());
        core.unserialize(&state).unwrap();
        assert_eq!(core.save_ram().unwrap()[123], 0x5a);
        if playlist {
            assert!(core.set_disc(0));
            let previous = core.serialize().unwrap();
            core.unserialize(&previous).unwrap();
            core.want_disc(1);
            core.settle_disc();
            assert_eq!(core.discs(), Some((1, 2)));
        }
    }
}

#[test]
fn six_button_shoulders_remain_game_buttons_and_cd_rewind_is_disabled() {
    let d = common::tmp_root_with_carts(&[]);
    std::fs::write(d.path().join("Games/PCE/Test.pce"), [0; 8192]).unwrap();
    let session = common::session_playing(d.path());
    assert_eq!(session.app().core(), Core::BeetlePceFast);
    assert!(!session.app().taken_buttons().contains(&Btn::L1));
    assert!(!session.app().taken_buttons().contains(&Btn::R1));
    let mut app = slot::app::App::new(Vec::new());
    app.set_platform(Platform::PceCd);
    assert!(!app.records_rewind());
}
