mod common;
use slot::app::Phase;
use slot_input::{Action, Btn};
use slot_store::{Core, Platform, Shader};

#[test]
fn archives_stay_intact_bios_is_hidden_and_titles_are_searchable() {
    let d = common::tmp_root_with_carts(&[]);
    let games = d.path().join("Games/NEOGEO");
    for name in ["mslug.zip", "mslug.7z", "kof98.7z", "neogeo.zip", "NEOGEO.7z", "loose.bin"] {
        std::fs::write(games.join(name), "archive not unpacked during scan").unwrap();
    }
    std::fs::write(d.path().join("Labels/NEOGEO/mslug.png"), "label").unwrap();
    let carts = slot_store::scan(d.path()).unwrap();
    assert_eq!(carts.len(), 2);
    let hits = slot_store::library::search(&carts, "metal slug");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].stem, "mslug");
    assert!(hits[0].display_name().starts_with("Metal Slug"));
    assert_eq!(hits[0].rom, games.join("mslug.zip"));
    assert!(hits[0].label.as_ref().unwrap().ends_with("Labels/NEOGEO/mslug.png"));
    assert_eq!(slot_store::library::search(&carts, "kof98").len(), 1);
    assert_eq!(slot_store::library::search(&carts, "king fighters").len(), 1);
    assert_eq!(Core::default_for(Platform::NeoGeo), Core::FbNeo);
    assert!(Core::FbNeo.runs(Platform::NeoGeo));
    assert!(!Core::FbNeo.runs(Platform::Ngp));
    assert!(!Platform::NeoGeo.takes_archives());
}

#[test]
fn missing_bios_returns_to_shelf_and_settings_are_independent() {
    let d = common::tmp_root_with_carts(&[]);
    std::fs::write(d.path().join("Games/NEOGEO/mslug.zip"), "archive").unwrap();
    common::clocked(d.path());
    let mut session = slot::session::Session::boot(d.path().into());
    common::open_shelf(session.app_mut());
    session.app_mut().apply(Action::Insert);
    for _ in 0..120 { session.update(1.0 / 60.0); }
    assert!(matches!(session.app().phase(), Phase::Shelf));
    assert!(!session.has_core());
    common::open_at(session.app_mut(), slot_ui::QuickRow::NeoGeoShader);
    common::press(session.app_mut(), Btn::Right);
    let state = slot_store::read_slot_state(d.path());
    assert_eq!(state.shader_neogeo, Shader::Grid);
    assert_eq!(state.shader_ngp, Shader::Off);
    assert_eq!(state.shader_pce, Shader::Off);
    for dir in ["Games", "Labels", "Saves", "States", "Overlays"] {
        assert!(d.path().join(dir).join("NEOGEO").is_dir());
    }
}

#[test]
#[ignore = "requires native FBNeo and make-neogeo.py diagnostic fixture"]
fn native_core_runs_restores_and_rejects_invalid_sets() {
    use slot_retro::ButtonMask;
    let root = std::path::PathBuf::from(std::env::var_os("SLOT_TEST_NEOGEO_ROOT").unwrap());
    for _ in 0..2 {
        let opened = slot::core::open_core(&root, Core::FbNeo, "", false, None, None);
        assert!(opened.named);
        let mut core = opened.core;
        core.load(&root.join("Games/NEOGEO/nam1975.zip")).unwrap();
        assert_eq!(core.canvas(), (320, 240));
        for _ in 0..120 { core.run_frame(ButtonMask::default()); core.take_audio(); }
        assert!(core.video_xrgb8888().chunks_exact(4).any(|p| p[1] > 100), "diagnostic green backdrop");
        let state = core.serialize().unwrap();
        assert!(state.len() > 100_000);
        core.run_frame(ButtonMask::default());
        core.unserialize(&state).unwrap();
    }
    assert!(root.join("Saves/NEOGEO/fbneo/nam1975.fs").is_file());
    for name in ["unknown.zip", "mslug.zip"] {
        let opened = slot::core::open_core(&root, Core::FbNeo, "", false, None, None);
        assert!(opened.named);
        let mut core = opened.core;
        std::fs::copy(root.join("BIOS/neogeo.zip"), root.join("Games/NEOGEO").join(name)).unwrap();
        assert!(core.load(&root.join("Games/NEOGEO").join(name)).is_err());
    }
}
