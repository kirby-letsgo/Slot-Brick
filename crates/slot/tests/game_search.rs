mod common;

use slot::app::{App, Phase};
use slot_input::{Action, Btn};
use slot_store::{Cart, Platform, SlotState};
use slot_ui::{QuickRow, TexId};

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
fn library() -> App {
    App::new(vec![
        cart(Platform::Gba, "Alpha"),
        cart(Platform::Md, "Sonic"),
        cart(Platform::Gg, "Sonic"),
        cart(Platform::Sms, "Sonic"),
        cart(Platform::Gb, "Zelda"),
    ])
}
fn tap(app: &mut App, btn: Btn) {
    app.apply(Action::GbaDown(btn));
    app.apply(Action::GbaUp(btn));
}

#[test]
fn every_systems_matching_carts_share_one_shelf_and_keep_their_identity() {
    let mut app = library();
    app.search_games("sONic");
    assert_eq!(app.game_search_text(), Some(("sONic", 3)));
    for platform in [Platform::Md, Platform::Gg, Platform::Sms] {
        let selected = app.selected_cart().unwrap();
        assert_eq!(selected.platform, platform);
        assert_eq!(app.browsed_rom(), Some(selected.rom.as_path()));
        assert_eq!(app.slot_text().as_deref(), Some(platform.name()));
        assert!(app.title_card_text().unwrap().contains(platform.name()));
        tap(&mut app, Btn::R1); // System shoulders must not leave the combined results.
        assert_eq!(app.selected_cart().unwrap().platform, platform);
        tap(&mut app, Btn::Right);
    }
    tap(&mut app, Btn::B);
    assert_eq!(app.game_search_text(), None);
    assert_eq!(app.selected_stem(), Some("Alpha"));
    assert_eq!(app.carts().count(), 5);
}

#[test]
fn all_words_match_in_any_order_ignoring_case_and_separators() {
    let carts = vec![
        cart(Platform::Gg, "Sonic_The_Hedgehog (USA)"),
        cart(Platform::Sms, "Sonic - The Hedgehog 2"),
        cart(Platform::Md, "Sonic Spinball"),
        cart(Platform::Gb, "Pokémon Pinball"),
    ];
    assert_eq!(
        slot_store::library::search(&carts, "  HEDGE sonic ").len(),
        2
    );
    assert_eq!(
        slot_store::library::search(&carts, "sonic 2")[0].platform,
        Platform::Sms
    );
    assert_eq!(slot_store::library::search(&carts, "POKÉMON").len(), 1);
    assert!(slot_store::library::search(&carts, "missing").is_empty());
    assert_eq!(slot_store::library::search(&carts, "  ").len(), 4);
}

#[test]
fn no_results_can_be_edited_or_left_and_cannot_launch_a_stale_cart() {
    let mut app = library();
    app.search_games("missing");
    assert!(app.selected_cart().is_none());
    assert_eq!(app.slot_text().as_deref(), Some("NO MATCHES"));
    assert_eq!(app.game_search_text(), Some(("missing", 0)));
    tap(&mut app, Btn::A);
    tap(&mut app, Btn::Y);
    tap(&mut app, Btn::Right);
    assert!(matches!(app.phase(), Phase::Shelf));
    app.search_games("Sonic");
    assert_eq!(app.carts().count(), 3);
    tap(&mut app, Btn::B);
    assert_eq!(app.selected_stem(), Some("Alpha"));
}

#[test]
fn search_includes_non_favorites_and_preserves_the_normal_filter() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta"]);
    std::fs::write(d.path().join("Games/GG/Beta.gg"), [0; 32]).unwrap();
    slot_store::write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            favorites_only: true,
            ..SlotState::default()
        },
    )
    .unwrap();
    std::fs::write(
        d.path().join(slot_store::LIBRARY_FILE),
        "GBA\tAlpha\t0\t0\t1\n",
    )
    .unwrap();
    let (mut app, _) = common::app_booting_at(d.path(), 1_800_000_000);
    assert_eq!(app.carts().count(), 1);
    app.search_games("Beta");
    assert_eq!(app.carts().count(), 2);
    tap(&mut app, Btn::Right);
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Gg);
    tap(&mut app, Btn::Y);
    // Starring rebuilds a filtered library, but must keep this system's Beta selected.
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Gg);
    assert_eq!(app.carts().count(), 2);
    tap(&mut app, Btn::B);
    assert_eq!(app.selected_stem(), Some("Alpha"));
    let games: Vec<_> = app.carts().map(|c| (c.platform, c.stem.as_str())).collect();
    assert_eq!(games, [(Platform::Gba, "Alpha"), (Platform::Gg, "Beta")]);
}

#[test]
fn a_same_named_result_launches_and_resumes_in_its_own_system() {
    let mut app = library();
    app.search_games("Sonic");
    tap(&mut app, Btn::Right);
    tap(&mut app, Btn::Right);
    tap(&mut app, Btn::A);
    assert!(matches!(app.phase(), Phase::Inserting { .. }));
    assert_eq!(app.seated_cart().unwrap().platform, Platform::Sms);
    assert_eq!(app.seated_cart().unwrap().rom.extension().unwrap(), "sms");
    app.apply(Action::Eject);
    for _ in 0..180 {
        app.update(1.0 / 60.0);
    }
    assert!(matches!(app.phase(), Phase::Shelf));
    assert_eq!(app.game_search_text(), Some(("Sonic", 3)));
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Sms);
}

#[test]
fn artwork_moves_with_its_system_and_search_changes_invalidate_pending_positions() {
    let mut app = library();
    for (i, platform) in [Platform::Md, Platform::Gg, Platform::Sms]
        .into_iter()
        .enumerate()
    {
        app.set_cart_face(platform.index(), 0, TexId::from_raw(i + 10));
    }
    let revision = app.face_revision();
    app.search_games("Sonic");
    assert_ne!(app.face_revision(), revision);
    // All three supplied faces were transferred; no result needs rebuilding.
    assert!(app.face_wants().is_empty());
    app.search_games("Zelda");
    assert_eq!(app.face_wants().len(), 1);
    assert_eq!(app.shed_faces().len(), 3);
}

#[cfg(feature = "brick")]
#[test]
fn menu_and_x_open_an_offline_keyboard_cancel_preserves_results_and_empty_shows_all() {
    let mut app = library();
    app.apply(Action::QuickMenu);
    tap(&mut app, Btn::Down);
    tap(&mut app, Btn::Down);
    tap(&mut app, Btn::A);
    assert!(matches!(
        app.phase(),
        Phase::QuickMenu {
            row: QuickRow::LibrarySearch
        }
    ));
    tap(&mut app, Btn::A);
    assert!(app.overlay_open());
    tap(&mut app, Btn::Start); // Empty query = all games.
    assert!(!app.overlay_open());
    assert_eq!(app.game_search_text(), Some(("", 5)));
    app.search_games("Sonic");
    tap(&mut app, Btn::Right);
    tap(&mut app, Btn::X);
    assert!(app.overlay_open());
    tap(&mut app, Btn::X); // Delete in editor, then cancel.
    tap(&mut app, Btn::B);
    assert_eq!(app.game_search_text(), Some(("Sonic", 3)));
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Gg);
    tap(&mut app, Btn::X);
    tap(&mut app, Btn::Start); // Resubmit keeps selection.
    assert_eq!(app.selected_cart().unwrap().platform, Platform::Gg);
    tap(&mut app, Btn::B);
    tap(&mut app, Btn::X); // Direct shortcut on normal shelves, too.
    assert!(app.overlay_open());
}

#[test]
fn launching_from_search_uses_that_systems_saves_states_and_play_history() {
    use slot::session::Session;
    use slot_store::{Core, StateRing};
    let d = common::tmp_root_with_carts(&["Alpha"]);
    for (system, ext) in [("GG", "gg"), ("SMS", "sms")] {
        std::fs::write(
            d.path().join(format!("Games/{system}/Sonic.{ext}")),
            [0; 32],
        )
        .unwrap();
        std::fs::write(
            d.path().join(format!("Saves/{system}/Sonic.sav")),
            system.as_bytes(),
        )
        .unwrap();
    }
    common::clocked(d.path());
    let mut session = Session::boot(d.path().into());
    session.app_mut().search_games("Sonic");
    tap(session.app_mut(), Btn::Right);
    tap(session.app_mut(), Btn::A);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !matches!(session.app().phase(), Phase::Playing { .. }) {
        assert!(std::time::Instant::now() < deadline, "game did not launch");
        session.update(1.0 / 60.0);
        std::thread::yield_now();
    }
    session.app_mut().apply(Action::SaveState);
    session.app_mut().settle_saves();
    assert_eq!(
        std::fs::read(d.path().join("Saves/Backups/SMS/Sonic (1).sav")).unwrap(),
        b"SMS"
    );
    assert!(!d.path().join("Saves/Backups/GG/Sonic (1).sav").exists());
    assert!(
        !StateRing::new(d.path(), Platform::Sms, Core::GenesisPlusGx, "Sonic")
            .list()
            .unwrap()
            .is_empty()
    );
    assert!(
        StateRing::new(d.path(), Platform::Gg, Core::GenesisPlusGx, "Sonic")
            .list()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        slot_store::read_slot_state(d.path()).cart_platform,
        Some(Platform::Sms)
    );
    // The asynchronous history write must complete before inspecting its file.
    for _ in 0..100 {
        if slot_store::Library::read(d.path())
            .get(Platform::Sms, "Sonic")
            .last_played
            != 0
        {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let history = slot_store::Library::read(d.path());
    assert_ne!(history.get(Platform::Sms, "Sonic").last_played, 0);
    assert_eq!(history.get(Platform::Gg, "Sonic").last_played, 0);
}
