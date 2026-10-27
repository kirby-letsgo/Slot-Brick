mod common;

use slot::app::{App, Phase};
use slot_input::{Action, Btn};
use slot_store::{LibrarySort, Platform, SlotState};
use slot_ui::SystemEntry;

fn tap(app: &mut App, btn: Btn) {
    app.apply(Action::GbaDown(btn));
    app.apply(Action::GbaUp(btn));
}

fn fixture(favorites: bool) -> (tempfile::TempDir, App) {
    let d = common::tmp_root_with_carts(&["Alpha", "Sonic", "Never"]);
    for (system, ext) in [("GG", "gg"), ("SMS", "sms"), ("WS", "wsc"), ("PICO8", "p8")] {
        std::fs::write(d.path().join(format!("Games/{system}/Sonic.{ext}")), [0; 32]).unwrap();
    }
    slot_store::write_slot_state(d.path(), &SlotState {
        clock_set: true, favorites_only: favorites, library_sort: LibrarySort::MostPlayed,
        ..SlotState::default()
    }).unwrap();
    std::fs::write(d.path().join(slot_store::LIBRARY_FILE), concat!(
        "GBA\tAlpha\t9000\t10\t1\n",
        "GBA\tSonic\t30\t20\t0\n",
        "GG\tSonic\t20\t40\t0\n",
        "SMS\tSonic\t40\t30\t0\n",
        "WS\tSonic\t10\t50\t0\n",
        "PICO8\tSonic\t10\t60\t0\n",
        "GB\tRemoved\t999\t999\t1\n",
    )).unwrap();
    let (app, _) = common::app_booting_at(d.path(), 1_800_000_000);
    (d, app)
}

#[test]
fn recent_tile_opens_cross_system_history_newest_first_and_b_returns_to_it() {
    let (_d, mut app) = fixture(false);
    app.show_systems();
    // Navigate to the first tile regardless of the previously selected system.
    while app.system_grid().unwrap().selected != 0 { tap(&mut app, Btn::Left); }
    assert_eq!(app.system_grid().unwrap().entries[0], (SystemEntry::Recent, 6));
    tap(&mut app, Btn::A);
    assert!(!app.on_systems());
    assert!(matches!(app.phase(), Phase::Shelf), "opening a collection must not launch a game");
    assert_eq!(app.recent_games_count(), Some(6));
    assert_eq!(app.game_search_text(), None);
    assert_eq!(app.carts().map(|c| c.platform).collect::<Vec<_>>(),
        [Platform::Pico8, Platform::Ws, Platform::Gg, Platform::Sms, Platform::Gba, Platform::Gba]);
    for platform in [Platform::Pico8, Platform::Ws, Platform::Gg] {
        assert_eq!(app.selected_cart().unwrap().platform, platform);
        assert_eq!(app.slot_text().as_deref(), Some(platform.name()));
        assert!(app.title_card_text().unwrap().contains(platform.name()));
        tap(&mut app, Btn::R1);
        assert_eq!(app.selected_cart().unwrap().platform, platform);
        tap(&mut app, Btn::Right);
    }
    tap(&mut app, Btn::B);
    assert!(app.on_systems());
    let grid = app.system_grid().unwrap();
    assert_eq!(grid.entries[grid.selected].0, SystemEntry::Recent);
    tap(&mut app, Btn::A);
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Pico8);
}

#[test]
fn favorites_and_sort_changes_do_not_hide_history_or_change_its_order() {
    let (d, mut app) = fixture(true);
    let before = app.selected_cart().unwrap().clone();
    app.show_recent_games();
    let order: Vec<_> = app.carts().cloned().collect();
    tap(&mut app, Btn::Select);
    assert_eq!(app.carts().cloned().collect::<Vec<_>>(), order);
    tap(&mut app, Btn::Y);
    assert_eq!(app.recent_games_count(), Some(6));
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Pico8);
    tap(&mut app, Btn::B);
    assert!(app.on_systems());
    assert_eq!(app.selected_cart().unwrap().rom, before.rom);
    app.settle_saves();
    assert_eq!(slot_store::read_slot_state(d.path()).library_sort, LibrarySort::MostPlayed);
}

#[test]
fn search_from_recent_returns_to_the_same_game_then_the_grid() {
    let (_d, mut app) = fixture(false);
    app.show_recent_games();
    tap(&mut app, Btn::Right);
    let selected = app.selected_cart().unwrap().clone();
    app.search_games("Never"); // Global search also includes unplayed games.
    assert_eq!(app.game_search_text(), Some(("Never", 1)));
    assert_eq!(app.recent_games_count(), None);
    tap(&mut app, Btn::B);
    assert_eq!(app.recent_games_count(), Some(6));
    assert_eq!(app.selected_cart().unwrap().rom, selected.rom);
    app.apply(Action::QuickMenu);
    tap(&mut app, Btn::B);
    assert_eq!(app.recent_games_count(), Some(6));
    tap(&mut app, Btn::B);
    assert!(app.on_systems());
    app.search_games("missing");
    tap(&mut app, Btn::B);
    let grid = app.system_grid().unwrap();
    assert_eq!(grid.entries[grid.selected].0, SystemEntry::Recent);
}

#[test]
fn a_first_successful_launch_adds_history_and_eject_reorders_without_losing_identity() {
    let (_d, mut app) = fixture(false);
    app.show_recent_games();
    for _ in 0..3 { tap(&mut app, Btn::Right); }
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Sms);
    tap(&mut app, Btn::A);
    assert_eq!(app.seated_cart().unwrap().platform, Platform::Sms);
    app.on_core_ready();
    for _ in 0..180 { app.update(1.0 / 60.0); }
    assert!(matches!(app.phase(), Phase::Playing { .. }));
    app.apply(Action::Eject);
    for _ in 0..180 { app.update(1.0 / 60.0); }
    assert_eq!(app.recent_games_count(), Some(6));
    assert_eq!(app.carts().next().unwrap().platform, Platform::Sms);
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Sms);
    tap(&mut app, Btn::B);
    tap(&mut app, Btn::A);
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Sms);
}

#[test]
fn no_history_has_no_tile_and_an_empty_collection_cannot_launch_a_stale_cart() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta"]);
    common::clocked(d.path());
    let (mut app, _) = common::app_booting_at(d.path(), 1_800_000_000);
    app.show_systems();
    assert!(app.system_grid().unwrap().entries.iter().all(|(e, _)| *e != SystemEntry::Recent));
    app.show_recent_games();
    assert_eq!(app.recent_games_count(), Some(0));
    assert!(app.selected_cart().is_none());
    assert_eq!(app.slot_text().as_deref(), Some("NO RECENT GAMES"));
    for btn in [Btn::A, Btn::Y, Btn::Left, Btn::R1] { tap(&mut app, btn); }
    assert!(matches!(app.phase(), Phase::Shelf));
    tap(&mut app, Btn::B);
    tap(&mut app, Btn::A);
    tap(&mut app, Btn::A);
    app.on_core_ready();
    for _ in 0..180 { app.update(1.0 / 60.0); }
    app.apply(Action::Eject);
    for _ in 0..180 { app.update(1.0 / 60.0); }
    tap(&mut app, Btn::B);
    assert_eq!(app.system_grid().unwrap().entries[0], (SystemEntry::Recent, 1));
}

fn favorites_fixture() -> (tempfile::TempDir, App) {
    let d = common::tmp_root_with_carts(&["Alpha", "Never"]);
    for (system, ext) in [("MD", "md"), ("GG", "gg"), ("SMS", "sms")] {
        std::fs::write(d.path().join(format!("Games/{system}/Sonic.{ext}")), [0; 32]).unwrap();
    }
    slot_store::write_slot_state(d.path(), &SlotState {
        clock_set: true, library_sort: LibrarySort::MostPlayed,
        ..SlotState::default()
    }).unwrap();
    std::fs::write(d.path().join(slot_store::LIBRARY_FILE), concat!(
        "GBA\tAlpha\t40\t0\t1\n",
        "MD\tSonic\t300\t0\t1\n",
        "GG\tSonic\t100\t0\t1\n",
        "SMS\tSonic\t800\t0\t1\n",
        "GB\tMissing\t0\t0\t1\n",
    )).unwrap();
    let (app, _) = common::app_booting_at(d.path(), 1_800_000_000);
    (d, app)
}

#[test]
fn favorites_tile_opens_only_starred_games_across_systems_and_returns_to_grid() {
    let (_d, mut app) = favorites_fixture();
    let before = app.selected_cart().unwrap().clone();
    app.show_systems();
    let grid = app.system_grid().unwrap();
    let favorite = grid.entries.iter().position(|(entry, _)| *entry == SystemEntry::Favorites).unwrap();
    assert_eq!(grid.entries[favorite], (SystemEntry::Favorites, 4));
    while app.system_grid().unwrap().selected != favorite {
        tap(&mut app, Btn::Left);
    }
    tap(&mut app, Btn::A);
    assert!(matches!(app.phase(), Phase::Shelf));
    assert!(!app.on_systems());
    assert_eq!(app.favorite_games_count(), Some(4));
    assert_eq!(app.recent_games_count(), None);
    assert_eq!(app.game_search_text(), None);
    assert_eq!(app.carts().map(|c| (c.platform, c.stem.as_str())).collect::<Vec<_>>(),
        [(Platform::Gba, "Alpha"), (Platform::Md, "Sonic"),
         (Platform::Gg, "Sonic"), (Platform::Sms, "Sonic")]);
    for platform in [Platform::Gba, Platform::Md, Platform::Gg, Platform::Sms] {
        assert_eq!(app.selected_cart().unwrap().platform, platform);
        assert!(app.title_card_text().unwrap().contains(platform.name()));
        tap(&mut app, Btn::R1);
        assert_eq!(app.selected_cart().unwrap().platform, platform);
        tap(&mut app, Btn::Right);
    }
    tap(&mut app, Btn::B);
    assert!(app.on_systems());
    let grid = app.system_grid().unwrap();
    assert_eq!(grid.entries[grid.selected].0, SystemEntry::Favorites);
    assert_eq!(app.selected_cart().unwrap().rom, before.rom);
    tap(&mut app, Btn::A);
    assert_eq!(app.selected_cart().unwrap().stem, "Alpha");
}

#[test]
fn removing_last_favorite_clears_collection_and_tile_without_a_stale_launch() {
    let (_d, mut app) = fixture(true);
    app.show_systems();
    assert!(app.system_grid().unwrap().entries.iter().any(|(e,_)| *e == SystemEntry::Favorites));
    // Even with the ordinary Favorites-only filter, removing the last star cannot
    // make the dedicated collection display every game.
    app.show_favorite_games();
    assert_eq!(app.favorite_games_count(), Some(1));
    assert_eq!(app.selected_stem(), Some("Alpha"));
    tap(&mut app, Btn::Y);
    assert_eq!(app.favorite_games_count(), Some(0));
    assert!(app.selected_cart().is_none());
    assert_eq!(app.slot_text().as_deref(), Some("NO FAVORITES"));
    for btn in [Btn::A, Btn::Right, Btn::R1] { tap(&mut app, btn); }
    assert!(matches!(app.phase(), Phase::Shelf));
    tap(&mut app, Btn::B);
    assert!(app.on_systems());
    assert!(app.system_grid().unwrap().entries.iter().all(|(e,_)| *e != SystemEntry::Favorites));
    assert!(app.selected_cart().is_some());
}

#[test]
fn searching_from_favorites_restores_the_same_platform_then_grid() {
    let (_d, mut app) = favorites_fixture();
    app.show_favorite_games();
    tap(&mut app, Btn::Right);
    tap(&mut app, Btn::Right);
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Gg);
    app.search_games("Never");
    assert_eq!(app.game_search_text(), Some(("Never", 1)));
    assert_eq!(app.favorite_games_count(), None);
    tap(&mut app, Btn::B);
    assert_eq!(app.favorite_games_count(), Some(4));
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Gg);
    app.apply(Action::QuickMenu);
    tap(&mut app, Btn::B);
    assert_eq!(app.favorite_games_count(), Some(4));
    tap(&mut app, Btn::B);
    assert!(app.on_systems());
    assert_eq!(app.system_grid().unwrap().entries[app.system_grid().unwrap().selected].0,
        SystemEntry::Favorites);
}

#[test]
fn playing_from_favorites_uses_the_matching_games_system_and_returns_to_it() {
    let (_d, mut app) = favorites_fixture();
    app.show_favorite_games();
    tap(&mut app, Btn::Right);
    tap(&mut app, Btn::Right);
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Gg);
    tap(&mut app, Btn::A);
    assert!(matches!(app.phase(), Phase::Inserting { .. }));
    assert_eq!(app.seated_cart().unwrap().platform, Platform::Gg);
    app.apply(Action::Eject);
    for _ in 0..180 { app.update(1.0 / 60.0); }
    assert_eq!(app.favorite_games_count(), Some(4));
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Gg);
}

#[test]
fn timestamps_tie_by_title_then_system_and_removed_games_stay_out() {
    let (_d, app) = fixture(false);
    let carts: Vec<_> = app.carts().cloned().collect();
    let history = slot_store::Library::parse("GBA\tSonic\t1\t10\t0\nGG\tSonic\t1\t10\t0\nGBA\tAlpha\t1\t10\t0\nGB\tGone\t1\t20\t0\n");
    let result = slot_store::library::recent(&carts, &history);
    assert_eq!(result.iter().map(|c| (c.platform, c.stem.as_str())).collect::<Vec<_>>(),
        [(Platform::Gba, "Alpha"), (Platform::Gba, "Sonic"), (Platform::Gg, "Sonic")]);
}
