mod common;

use std::fs;
use slot::{app::{App,Phase},dreamcast};
use slot_store::{Core,Platform};
use slot_ui::SystemEntry;

fn fixture()->tempfile::TempDir{
    let d=common::tmp_root_with_carts(&[]);
    fs::write(d.path().join("Games/DC/First.chd"), b"pretend compressed disc").unwrap();
    fs::create_dir_all(d.path().join("Games/DC/Multi")).unwrap();
    fs::write(d.path().join("Games/DC/Multi/Together.m3u"), b"Disc 1.gdi\nDisc 2.gdi\n").unwrap();
    fs::write(d.path().join("Games/DC/Multi/Disc 1.gdi"), b"1\n1 0 4 2352 track01.bin 0\n").unwrap();
    fs::write(d.path().join("Games/DC/Multi/Disc 2.gdi"), b"1\n1 0 4 2352 track02.bin 0\n").unwrap();
    fs::write(d.path().join("Games/DC/Multi/track01.bin"), b"track").unwrap();
    fs::write(d.path().join("Games/DC/Multi/track02.bin"), b"track").unwrap();
    common::clocked(d.path());
    d
}
#[test]
fn dreamcast_is_an_opt_in_disc_shelf_with_no_track_duplication(){
    let d=fixture();
    let carts=slot_store::scan(d.path()).unwrap();
    let dc:Vec<_>=carts.iter().filter(|c|c.platform==Platform::Dc).collect();
    assert_eq!(dc.len(),2,"M3U should hide both GDI disc entries");
    assert!(dc.iter().any(|c|c.stem=="First"));
    assert!(dc.iter().any(|c|c.stem=="Together"));
    assert!(Platform::Dc.is_disc());
    assert!(!Platform::Dc.takes_archives());
    assert_eq!(Core::default_for(Platform::Dc),Core::Flycast);
    assert!(Core::Flycast.runs(Platform::Dc));
    assert!(!Core::Flycast.runs(Platform::Ps1));

    let mut app=App::new(carts);
    app.show_systems();
    let grid=app.system_grid().unwrap();
    assert!(grid.entries.iter().any(|(e,n)|*e==SystemEntry::Platform(Platform::Dc) && *n==2));
    app.search_games("Together");
    assert_eq!(app.game_search_text(),Some(("Together",1)));
    assert_eq!(app.selected_cart().unwrap().platform,Platform::Dc);
}
#[test]
fn dreamcast_missing_runtime_is_safe_and_does_not_exit_slot(){
    let d=fixture();
    let carts=slot_store::scan(d.path()).unwrap();
    let chosen=carts.iter().find(|c|c.platform==Platform::Dc).unwrap();
    assert!(dreamcast::prepare(d.path(),chosen).unwrap_err().contains("Flycast"));
    let setup=slot::system_setup::scan(d.path(),&carts);
    let item=setup.iter().find(|e|e.name==Platform::Dc.name()).unwrap();
    assert_eq!(item.status,slot_ui::SetupStatus::Missing);
}
#[test]
fn native_exit_returns_to_dreamcast_shelf_without_autolaunching(){
    let d=fixture();
    let mut state=slot_store::read_slot_state(d.path());
    state.cart=Some("First".into());state.cart_platform=Some(Platform::Dc);
    slot_store::write_slot_state(d.path(),&state).unwrap();
    let app=App::boot(d.path());
    assert!(matches!(app.phase(),Phase::Shelf));
    assert_eq!(app.selected_stem(),Some("First"));
    assert_eq!(app.selected_cart().unwrap().platform,Platform::Dc);
}
#[test]
fn standalone_fps_setting_is_available_but_libretro_overrides_are_not(){
    let d=fixture();
    let mut app=App::boot(d.path());
    app.search_games("First");
    assert_eq!(app.selected_cart().unwrap().platform,Platform::Dc);
    // Dreamcast is native; only its own Flycast FPS flag is configurable from Slot.
    app.apply(slot_input::Action::GbaDown(slot_input::Btn::Start));
    let options=app.cart_menu().expect("Dreamcast disc uses existing cart options");
    assert_eq!(options.0,slot_ui::CartRow::EmulatorSettings);
    app.apply(slot_input::Action::GbaDown(slot_input::Btn::A));
    let view=app.game_settings_view().expect("settings open");
    assert_eq!(view.rows.len(),2);
    assert_eq!(view.rows[0].0,"SHOW FPS");
}
