mod common;

use std::path::Path;
use std::time::Duration;

use common::{app_booting_at, tmp_root_with_carts, StubSnapshot, CLOCK_IS_SET};
use slot::app::{App, Phase};
use slot_input::{Action, Btn};
use slot_store::{read_slot_state, write_slot_state, SlotState};
use slot_ui::{QuickRow, QuickValue};

const MIN: Duration = Duration::from_secs(60);

fn playing(root: &Path, auto_sleep_min: u8) -> App {
    write_slot_state(
        root,
        &SlotState {
            cart: Some("Emerald".into()),
            clock_set: true,
            auto_sleep_min,
            ..Default::default()
        },
    )
    .expect("write slot.state");
    let mut a = App::boot(root);
    a.set_snapshot(StubSnapshot::boxed());
    a.on_core_ready();
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    assert!(
        matches!(a.phase(), Phase::Playing { .. }),
        "{:?}",
        a.phase()
    );
    a
}

fn on_shelf(root: &Path, auto_sleep_min: u8) -> App {
    write_slot_state(
        root,
        &SlotState {
            clock_set: true,
            auto_sleep_min,
            ..Default::default()
        },
    )
    .expect("write slot.state");
    let (a, _clock) = app_booting_at(root, CLOCK_IS_SET);
    assert!(matches!(a.phase(), Phase::Shelf), "{:?}", a.phase());
    a
}

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

/// HOME → Gameplay → Auto Sleep.
fn open_auto_sleep(a: &mut App) {
    a.apply(Action::QuickMenu);
    for row in [QuickRow::Game, QuickRow::AutoSleep] {
        for _ in 0..row.position() {
            press(a, Btn::Down);
        }
        if row == QuickRow::Game {
            press(a, Btn::A);
        }
    }
    assert_eq!(a.quick_menu(), Some(QuickRow::AutoSleep));
}

#[test]
fn by_default_a_game_sleeps_after_15_minutes_and_the_shelf_after_5() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let default = SlotState::default().auto_sleep_min;
    assert_eq!(
        playing(d.path(), default).auto_sleep_after(),
        Some(15 * MIN)
    );
    assert_eq!(
        on_shelf(d.path(), default).auto_sleep_after(),
        Some(5 * MIN)
    );
}

#[test]
fn a_game_waits_the_chosen_time_and_the_shelf_never_more_than_5_minutes() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    for (min, game, shelf) in [(2, 2, 2), (5, 5, 5), (10, 10, 5), (30, 30, 5)] {
        assert_eq!(
            playing(d.path(), min).auto_sleep_after(),
            Some(game * MIN),
            "{min} min, in a game"
        );
        assert_eq!(
            on_shelf(d.path(), min).auto_sleep_after(),
            Some(shelf * MIN),
            "{min} min, on the shelf"
        );
    }
}

#[test]
fn off_never_sleeps() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    assert_eq!(playing(d.path(), 0).auto_sleep_after(), None);
    assert_eq!(on_shelf(d.path(), 0).auto_sleep_after(), None);
}

#[test]
fn gameplay_auto_sleep_steps_through_its_times_and_is_saved() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut a = on_shelf(d.path(), 15);
    open_auto_sleep(&mut a);
    assert_eq!(a.quick_value(QuickRow::AutoSleep), Some(QuickValue::Min15));
    let mut seen = Vec::new();
    for _ in 0..6 {
        press(&mut a, Btn::Right);
        seen.push(a.quick_value(QuickRow::AutoSleep).expect("no value"));
    }
    assert_eq!(
        seen,
        [
            QuickValue::Min30,
            QuickValue::Off,
            QuickValue::Min2,
            QuickValue::Min5,
            QuickValue::Min10,
            QuickValue::Min15
        ]
    );
    press(&mut a, Btn::Left);
    assert_eq!(read_slot_state(d.path()).auto_sleep_min, 10);
    assert_eq!(
        a.auto_sleep_after(),
        Some(5 * MIN),
        "the menus waited longer than 5 minutes"
    );
}

#[test]
fn the_old_disable_file_turns_it_off_and_keeps_the_row_at_off() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    std::fs::write(d.path().join("DISABLE_AUTO_SLEEP"), "").expect("write the file");
    let mut a = on_shelf(d.path(), 15);
    assert_eq!(a.auto_sleep_after(), None);
    open_auto_sleep(&mut a);
    assert_eq!(a.quick_value(QuickRow::AutoSleep), Some(QuickValue::Off));
    press(&mut a, Btn::Right);
    assert_eq!(a.quick_value(QuickRow::AutoSleep), Some(QuickValue::Off));
    assert_eq!(
        read_slot_state(d.path()).auto_sleep_min,
        15,
        "the setting changed under the file"
    );
}
