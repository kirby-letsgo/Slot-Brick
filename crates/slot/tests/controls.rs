#![cfg(feature = "brick")]
mod common;
use slot::app::Phase;
use slot::input::brick::button_for;
use slot_input::{Action, Btn, RawEvent};
use slot_store::{
    controls::{Controls, Shortcut},
    Core, Platform, StateRing,
};
use slot_ui::{QuickRow, QuickValue};

#[test]
fn brick_and_pro_function_keys_do_not_confuse_stick_clicks() {
    assert_eq!(
        [button_for(9, false), button_for(10, false)],
        [Some(Btn::F1), Some(Btn::F2)]
    );
    assert_eq!([button_for(9, true), button_for(10, true)], [None, None]);
    assert_eq!(
        [button_for(11, true), button_for(12, true)],
        [Some(Btn::F1), Some(Btn::F2)]
    );
    assert_eq!([button_for(11, false), button_for(12, false)], [None, None]);
}

#[test]
fn default_f1_searches_without_wifi_and_function_keys_do_not_edit_text() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta"]);
    let mut app = common::boot(d.path());
    app.apply(Action::GbaDown(Btn::F1));
    assert!(app.overlay_open());
    app.apply(Action::GbaDown(Btn::F1));
    app.apply(Action::GbaDown(Btn::F2));
    app.apply(Action::GbaDown(Btn::Start));
    assert_eq!(app.game_search_text(), Some(("", 2)));
}

#[test]
fn menu_assignments_persist_and_take_effect_after_reboot() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta"]);
    let mut app = common::boot(d.path());
    common::open_at(&mut app, QuickRow::F1Shelf);
    assert_eq!(
        app.quick_value(QuickRow::F1Shelf),
        Some(QuickValue::KeySearch)
    );
    app.apply(Action::GbaDown(Btn::Right));
    assert_eq!(
        app.quick_value(QuickRow::F1Shelf),
        Some(QuickValue::KeyFavorite)
    );
    assert_eq!(Controls::read(d.path()).0[0], Shortcut::Favorite);
    assert_eq!(Controls::read(d.path()).0[2], Shortcut::SaveState);
    // F1 cannot toggle a favorite while its settings menu is open.
    app.apply(Action::GbaDown(Btn::F1));
    assert_eq!(app.quick_menu(), Some(QuickRow::F1Shelf));
    let mut rebooted = common::boot(d.path());
    assert_eq!(
        rebooted.quick_value(QuickRow::F1Shelf),
        Some(QuickValue::KeyFavorite)
    );
    rebooted.apply(Action::GbaDown(Btn::F1));
    assert!(!rebooted.overlay_open());
}

#[test]
fn default_f1_saves_and_f2_opens_the_state_chooser_through_real_input() {
    let d = common::tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut session = common::session_playing(d.path());
    let ring = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    assert!(ring.list().unwrap().is_empty());
    session.feed([RawEvent::Down(Btn::F1), RawEvent::Up(Btn::F1)], 100);
    session.app_mut().settle_saves();
    assert_eq!(ring.list().unwrap().len(), 1);
    session.feed([RawEvent::Down(Btn::F2), RawEvent::Up(Btn::F2)], 200);
    assert!(matches!(session.app().phase(), Phase::Polaroids { .. }));
    session.feed([RawEvent::Down(Btn::B), RawEvent::Up(Btn::B)], 300);
    assert!(matches!(session.app().phase(), Phase::Playing { .. }));
}

#[test]
fn off_keys_do_nothing_and_a_remapped_game_key_opens_the_pause_menu() {
    let d = common::tmp_root_with_carts(&["Emerald", "Fusion"]);
    Controls([Shortcut::Off, Shortcut::Off, Shortcut::Menu, Shortcut::Off])
        .write(d.path())
        .unwrap();
    let mut shelf = common::boot(d.path());
    shelf.apply(Action::GbaDown(Btn::F1));
    assert!(!shelf.overlay_open());
    let mut session = common::session_playing(d.path());
    session.feed([RawEvent::Down(Btn::F2), RawEvent::Up(Btn::F2)], 100);
    assert!(session.app().pause_menu().is_none());
    session.feed([RawEvent::Down(Btn::F1), RawEvent::Up(Btn::F1)], 200);
    assert_eq!(session.app().pause_menu(), Some(0));
}
