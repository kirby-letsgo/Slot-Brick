//! The Mega Drive shelf, its carts and its Screen page, as the app sees them.
mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use slot::app::{App, Phase};
use slot::session::Session;
use slot::video_mode::{video_mode_for_platform, VideoMode};
use slot_input::{Action, Btn, RawEvent};
use slot_store::{write_slot_state, Platform, SlotState};

fn session_playing(root: &Path) -> Session {
    common::clocked(root);
    let mut s = Session::boot(root.to_path_buf());
    s.feed([RawEvent::Down(Btn::A)], 16);
    s.feed([RawEvent::Up(Btn::A)], 32);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !matches!(s.app().phase(), Phase::Playing { .. }) {
        assert!(Instant::now() < deadline, "the cart never seated");
        s.update(1.0 / 60.0);
        std::thread::sleep(Duration::from_millis(1));
    }
    s
}

#[allow(dead_code)]
fn booted_on_shelf(d: tempfile::TempDir) -> (tempfile::TempDir, App) {
    write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    let app = App::boot(d.path());
    (d, app)
}

#[allow(dead_code)]
fn press(app: &mut App, btn: Btn) {
    app.apply(Action::GbaDown(btn));
}

#[test]
fn a_mega_drive_game_keeps_l_and_r_and_switches_its_picture_with_select_x() {
    let d = common::tmp_root_with_carts(&[]);
    std::fs::write(
        d.path().join("Games/MD/Sonic the Hedgehog.md"),
        [0u8; 0x200],
    )
    .unwrap();
    let mut s = session_playing(d.path());
    assert_eq!(
        s.app().canvas(),
        Platform::Md.canvas(),
        "the Mega Drive cart is not the one playing"
    );
    assert!(
        s.app().taken_buttons().is_empty(),
        "slot kept L1/R1 (X and Z) from a Mega Drive game"
    );
    let start = s.app().source_rect();
    for (t, e) in [
        RawEvent::Down(Btn::Select),
        RawEvent::Down(Btn::X),
        RawEvent::Up(Btn::X),
        RawEvent::Up(Btn::Select),
    ]
    .into_iter()
    .enumerate()
    {
        s.feed([e], 100 + 20 * t as u64);
    }
    assert_eq!(
        video_mode_for_platform(d.path(), "Sonic the Hedgehog", Platform::Md),
        VideoMode::Stretch
    );
    assert_eq!(s.app().toast(), Some(slot_ui::Toast::PictureFull));
    assert_ne!(s.app().source_rect(), start);
}
