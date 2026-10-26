mod common;

use slot::app::{App, Phase};
use slot_input::{Action, Btn};
use slot_store::{Cart, Platform, SlotState};

fn cart(platform: Platform, stem: &str) -> Cart {
    Cart {
        platform,
        stem: stem.into(),
        rom: format!(
            "/games/{}/{stem}.{}",
            platform.dir_name(),
            platform.extensions()[0]
        )
        .into(),
        label: None,
        famicom_label: None,
        title: String::new(),
        code: String::new(),
        shell: None,
    }
}
fn tap(app: &mut App, btn: Btn) {
    app.apply(Action::GbaDown(btn));
    app.apply(Action::GbaUp(btn));
}

#[test]
fn boot_opens_grid_and_a_only_opens_a_shelf() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta"]);
    slot_store::write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            ..SlotState::default()
        },
    )
    .unwrap();
    let mut app = App::boot(d.path());
    assert!(app.on_systems());
    assert!(app.face_wants().is_empty());
    assert!(app.browsed_rom().is_none());
    assert_eq!(app.system_grid().unwrap().entries, vec![(Platform::Gba, 2)]);
    tap(&mut app, Btn::A);
    assert!(!app.on_systems());
    assert!(matches!(app.phase(), Phase::Shelf));
    assert_eq!(app.selected_stem(), Some("Alpha"));
    assert!(app.browsed_rom().is_some());
    assert!(!app.face_wants().is_empty());
    tap(&mut app, Btn::Right);
    let selected = app.selected_stem().unwrap().to_owned();
    tap(&mut app, Btn::B);
    assert!(app.on_systems());
    tap(&mut app, Btn::A);
    assert_eq!(app.selected_stem(), Some(selected.as_str()));
}

#[test]
fn navigation_pages_and_restores_each_systems_cart() {
    let mut carts: Vec<_> = Platform::ALL
        .into_iter()
        .map(|p| cart(p, "First"))
        .collect();
    carts.push(cart(Platform::Gba, "Second"));
    let mut app = App::new(carts);
    tap(&mut app, Btn::Right);
    let selected = app.selected_stem().unwrap().to_owned();
    app.show_systems();
    assert_eq!(app.system_grid().unwrap().selected, 2);
    tap(&mut app, Btn::Down);
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Md);
    tap(&mut app, Btn::R1);
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Ngp);
    tap(&mut app, Btn::R1);
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Pico8);
    tap(&mut app, Btn::L1);
    // Return to the same cell in the first page.
    for _ in 0..5 { tap(&mut app, Btn::Right); }
    tap(&mut app, Btn::L1);
    tap(&mut app, Btn::Up);
    tap(&mut app, Btn::A);
    assert_eq!(app.selected_stem(), Some(selected.as_str()));
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Gba);
    assert!(matches!(app.phase(), Phase::Shelf));
}

#[test]
fn search_returns_to_its_origin_and_menus_keep_grid_selection() {
    let mut app = App::new(vec![
        cart(Platform::Gba, "Sonic"),
        cart(Platform::Md, "Sonic"),
    ]);
    app.show_systems();
    app.search_games("Sonic");
    assert!(!app.on_systems());
    assert_eq!(app.game_search_text(), Some(("Sonic", 2)));
    tap(&mut app, Btn::B);
    assert!(app.on_systems());
    let grid = app.system_grid();
    app.apply(Action::QuickMenu);
    assert!(!app.on_systems());
    tap(&mut app, Btn::B);
    assert_eq!(app.system_grid(), grid);
    tap(&mut app, Btn::A);
    app.search_games("Sonic");
    tap(&mut app, Btn::B);
    assert!(!app.on_systems());
}

#[test]
fn empty_library_is_safe_and_partial_pages_are_reachable() {
    let mut app = App::new(Vec::new());
    app.show_systems();
    for btn in [
        Btn::Left,
        Btn::Right,
        Btn::Up,
        Btn::Down,
        Btn::L1,
        Btn::R1,
        Btn::A,
        Btn::B,
    ] {
        tap(&mut app, btn);
    }
    assert!(app.on_systems());
    assert!(app.system_grid().unwrap().entries.is_empty());
    let mut app = App::new(
        Platform::ALL[..7]
            .iter()
            .map(|p| cart(*p, "Game"))
            .collect(),
    );
    app.show_systems();
    tap(&mut app, Btn::R1);
    assert_eq!(app.system_grid().unwrap().selected, 6);
    tap(&mut app, Btn::A);
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Ps1);
}

#[test]
fn grid_and_shelf_use_the_same_wallpaper_texture_and_scrim() {
    let mut app = App::new(vec![cart(Platform::Gba, "Game")]);
    app.set_wallpaper(slot_ui::TexId::from_raw(900));
    app.show_systems();
    let mut grid = Vec::new();
    app.draw(&mut grid);
    tap(&mut app, Btn::A);
    let mut shelf = Vec::new();
    app.draw(&mut shelf);
    assert_eq!(&grid[..2], &shelf[..2]);
}
