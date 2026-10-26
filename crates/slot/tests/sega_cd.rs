mod common;
use slot::app::Phase;
use slot_input::{Action, Btn};
use slot_store::{Core, Platform, Shader};

#[test]
fn scan_groups_discs_tracks_and_playlists_without_other_console_formats() {
    let d = tempfile::tempdir().unwrap();
    slot::root::ensure(d.path());
    let games = d.path().join("Games/SEGACD");
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
    assert!(carts.iter().all(|c| c.platform == Platform::SegaCd));
    assert_eq!(Core::default_for(Platform::SegaCd), Core::PicoDrive);
    assert!(Core::PicoDrive.runs(Platform::SegaCd));
    assert!(!Platform::SegaCd.takes_archives());
}

#[test]
fn missing_bios_returns_to_shelf_without_starting_a_core() {
    let d = common::tmp_root_with_carts(&[]);
    std::fs::write(d.path().join("Games/SEGACD/Game.iso"), "disc").unwrap();
    assert!(slot::sega_cd::check_bios(d.path()).unwrap_err().contains("bios_CD_U.bin"));
    common::clocked(d.path());
    let mut session = slot::session::Session::boot(d.path().into());
    common::open_shelf(session.app_mut());
    session.app_mut().apply(Action::Insert);
    for _ in 0..120 { session.update(1.0 / 60.0); }
    assert!(matches!(session.app().phase(), Phase::Shelf));
    assert!(!session.has_core());
    let bios = d.path().join("BIOS/bios_CD_U.bin");
    std::fs::write(&bios, [0; 1024]).unwrap();
    assert!(slot::sega_cd::check_bios(d.path()).is_err());
    std::fs::write(&bios, vec![0; 128 * 1024]).unwrap();
    assert!(slot::sega_cd::check_bios(d.path()).is_ok());
}

#[test]
fn disc_selection_labels_and_settings_are_separate_from_playstation() {
    let d = common::tmp_root_with_carts(&[]);
    let games = d.path().join("Games/SEGACD");
    std::fs::write(games.join("Other.iso"), "disc").unwrap();
    std::fs::write(games.join("Game.m3u"), "Disc 1.iso\nDisc 2.iso\n").unwrap();
    for n in [1, 2] { std::fs::write(games.join(format!("Disc {n}.iso")), "disc").unwrap(); }
    std::fs::write(d.path().join("Labels/SEGACD/Disc 2.png"), "art").unwrap();
    slot_store::ini::write(d.path(), slot::disc_picker::DISCS_FILE, "Game", "2").unwrap();
    common::clocked(d.path());
    let mut app = slot::app::App::boot(d.path());
    common::open_shelf(&mut app);
    app.apply(Action::Insert);
    let faces = app.disc_faces_wanted().unwrap();
    assert_eq!(faces.len(), 2);
    assert!(faces[1].0.label.as_ref().unwrap().ends_with("Labels/SEGACD/Disc 2.png"));
    common::press(&mut app, Btn::A);
    assert_eq!(app.chosen_disc(), Some(0), "does not inherit PS1's disc choice");
    assert_eq!(slot_store::ini::value(d.path(), slot::disc_picker::DISCS_FILE, "SEGACD/Game").as_deref(), Some("1"));
    assert_eq!(slot_store::ini::value(d.path(), slot::disc_picker::DISCS_FILE, "Game").as_deref(), Some("2"));

    let d = common::tmp_root_with_carts(&["One", "Two"]);
    common::clocked(d.path());
    let mut app = slot::app::App::boot(d.path());
    common::open_at(&mut app, slot_ui::QuickRow::SegaCdShader);
    common::press(&mut app, Btn::Right);
    let state = slot_store::read_slot_state(d.path());
    assert_eq!(state.shader_segacd, Shader::Grid);
    assert_eq!(state.shader_32x, Shader::Off);
    assert_eq!(state.shader_ps1, Shader::Off);
}

#[test]
#[ignore = "requires native PicoDrive and the generated Sega CD test card"]
fn native_cd_loads_disc_formats_keeps_bram_and_selects_a_disc() {
    use slot_retro::ButtonMask;
    let root = std::path::PathBuf::from(std::env::var_os("SLOT_TEST_SEGACD_ROOT").unwrap());
    slot::root::ensure(&root);
    let path = std::env::var_os("SLOT_TEST_SEGACD_CORE").unwrap();
    std::fs::copy(path, root.join("System/picodrive_libretro.so")).unwrap();
    let games = root.join("Games/SEGACD");
    let mut formats = vec![games.join("CD Test (Disc 1).iso"), games.join("CD Test (Disc 1).cue"), games.join("CD Test.m3u")];
    if let Some(chd) = std::env::var_os("SLOT_TEST_SEGACD_CHD") { formats.push(chd.into()); }
    for rom in formats {
        let opened = slot::core::open_sega_cd(&root);
        assert!(opened.named);
        let mut core = opened.core;
        let playlist = rom.extension().unwrap() == "m3u";
        if playlist { core.want_disc(1); }
        core.load(&rom).unwrap();
        core.settle_disc();
        if playlist { assert_eq!(core.discs(), Some((1, 2))); }
        for _ in 0..120 { core.run_frame(ButtonMask::default()); core.take_audio(); }
        assert_eq!(core.canvas(), (320, 240));
        assert!(core.video_xrgb8888().chunks_exact(4).any(|p| p[1] > 100 && p[0] < 50), "test BIOS draws green");
        let mut bram = core.save_ram().unwrap();
        assert_eq!(bram.len(), 8192);
        bram[123] = 0x5a;
        core.load_save_ram(&bram).unwrap();
        assert_eq!(core.save_ram().unwrap()[123], 0x5a);
        let state = core.serialize().unwrap();
        core.run_frame(ButtonMask::default());
        core.unserialize(&state).unwrap();
        assert_eq!(core.save_ram().unwrap()[123], 0x5a);
        if playlist {
            assert!(core.set_disc(0));
            let previous_disc = core.serialize().unwrap();
            core.unserialize(&previous_disc).unwrap();
            core.want_disc(1);
            core.settle_disc();
            assert_eq!(core.discs(), Some((1, 2)), "the chosen disc wins over the resumed state");
        }
    }
}

#[test]
fn playstation_keeps_its_existing_disc_choice_and_labels() {
    let d = common::tmp_root_with_carts(&[]);
    let games = d.path().join("Games/PS1");
    for n in [1, 2] { std::fs::write(games.join(format!("Disc {n}.iso")), "disc").unwrap(); }
    std::fs::write(games.join("Other.iso"), "disc").unwrap();
    std::fs::write(games.join("Game.m3u"), "Disc 1.iso\nDisc 2.iso\n").unwrap();
    std::fs::write(d.path().join("Labels/PS1/Disc 2.png"), "art").unwrap();
    slot_store::ini::write(d.path(), slot::disc_picker::DISCS_FILE, "Game", "2").unwrap();
    common::clocked(d.path());
    let mut app = slot::app::App::boot(d.path());
    common::open_shelf(&mut app);
    app.apply(Action::Insert);
    let faces = app.disc_faces_wanted().unwrap();
    assert!(faces[1].0.label.as_ref().unwrap().ends_with("Labels/PS1/Disc 2.png"));
    common::press(&mut app, Btn::A);
    assert_eq!(app.chosen_disc(), Some(1));
    assert_eq!(slot_store::ini::value(d.path(), slot::disc_picker::DISCS_FILE, "Game").as_deref(), Some("2"));
    assert_eq!(slot_store::ini::value(d.path(), slot::disc_picker::DISCS_FILE, "SEGACD/Game"), None);
}
