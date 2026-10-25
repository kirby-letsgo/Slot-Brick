//! The Master System shelf, its carts and its Screen page, as the app sees them.
mod common;

use common::{open_at, press};
use slot::app::{App, Phase};
use slot_input::Btn;
use slot_store::{read_slot_state, write_slot_state, Platform, Shader, SlotState};
use slot_ui::{QuickRow, QuickValue};

fn booted_on_shelf(d: tempfile::TempDir) -> (tempfile::TempDir, App) {
    write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    let (app, _) = common::app_booting_at(d.path(), common::CLOCK_IS_SET);
    assert!(matches!(app.phase(), Phase::Shelf), "{:?}", app.phase());
    (d, app)
}

#[test]
fn the_master_system_shader_is_set_from_its_own_page() {
    let (d, mut a) = booted_on_shelf(common::tmp_root_with_carts(&["Emerald", "Fusion"]));
    open_at(&mut a, QuickRow::SmsShader);
    assert_eq!(a.quick_value(QuickRow::SmsShader), Some(QuickValue::Off));
    press(&mut a, Btn::Right);
    assert_eq!(read_slot_state(d.path()).shader_sms, Shader::Grid);
    assert_eq!(
        read_slot_state(d.path()).shader_md,
        Shader::Off,
        "the Mega Drive took the Master System one"
    );
    assert_eq!(a.quick_value(QuickRow::SmsShader), Some(QuickValue::Grid));
}

#[test]
fn the_master_system_page_has_its_shader_and_overlay_and_wraps_between_them() {
    let (_d, mut a) = booted_on_shelf(common::tmp_root_with_carts(&["Emerald", "Fusion"]));
    open_at(&mut a, QuickRow::SmsShader);
    press(&mut a, Btn::Down);
    assert_eq!(a.quick_menu(), Some(QuickRow::SmsOverlay));
    press(&mut a, Btn::Down);
    assert_eq!(a.quick_menu(), Some(QuickRow::SmsShader));
    press(&mut a, Btn::B);
    assert_eq!(a.quick_menu(), Some(QuickRow::ScreenSms));
}

#[test]
fn the_master_system_overlay_row_steps_through_its_own_folder_and_saves_it() {
    let d = common::tmp_root_with_carts(&["Emerald", "Fusion"]);
    for (platform, f) in [(Platform::Sms, "CRT.png"), (Platform::Md, "Other.png")] {
        let dir = slot_store::overlays_dir(d.path(), platform);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(f), b"x").unwrap();
    }
    let (d, mut a) = booted_on_shelf(d);
    open_at(&mut a, QuickRow::SmsOverlay);
    assert_eq!(a.overlay_text(Platform::Sms), "Off");
    press(&mut a, Btn::Right);
    assert_eq!(
        read_slot_state(d.path()).overlay(Platform::Sms),
        Some("CRT.png")
    );
    assert_eq!(
        read_slot_state(d.path()).overlay(Platform::Md),
        None,
        "the Mega Drive took the Master System one"
    );
    assert_eq!(a.overlay_text(Platform::Sms), "CRT");
    press(&mut a, Btn::Right);
    assert_eq!(read_slot_state(d.path()).overlay(Platform::Sms), None);
}

#[test]
fn a_master_system_cart_has_its_own_shelf_canvas_and_local_label() {
    let d = common::tmp_root_with_carts(&[]);
    std::fs::write(d.path().join("Games/SMS/Sonic.SMS"), [0u8; 0x8000]).unwrap();
    std::fs::write(d.path().join("Games/SMS/Zzz.sms"), [0u8; 0x8000]).unwrap();
    let label = d.path().join("Labels/SMS/Sonic.png");
    std::fs::write(&label, b"\x89PNG\r\n\x1a\n").unwrap();
    let carts = slot_store::scan(d.path()).unwrap();
    assert_eq!(carts.len(), 2);
    assert_eq!(carts[0].platform, Platform::Sms);
    assert_eq!(carts[0].label.as_ref(), Some(&label));
    assert_eq!(
        slot_store::Core::default_for(Platform::Sms),
        slot_store::Core::GenesisPlusGx
    );
    let (_, app) = booted_on_shelf(d);
    assert_eq!(app.selected_stem(), Some("Sonic"));
}

#[test]
fn master_system_picture_choices_do_not_leak_from_other_systems() {
    use slot::video_mode::{video_mode_for_platform, write_video_mode, VideoMode};
    let d = common::tmp_root_with_carts(&[]);
    slot_store::ini::write(
        d.path(),
        slot::video_mode::VIDEO_MODE_FILE,
        "Sonic",
        "stretch",
    )
    .unwrap();
    write_video_mode(d.path(), Platform::Md, "Sonic", VideoMode::Stretch).unwrap();
    assert_eq!(
        video_mode_for_platform(d.path(), "Sonic", Platform::Sms),
        VideoMode::Actual
    );
    write_video_mode(d.path(), Platform::Sms, "Sonic", VideoMode::Stretch).unwrap();
    assert_eq!(
        video_mode_for_platform(d.path(), "Sonic", Platform::Sms),
        VideoMode::Stretch
    );
}

#[test]
fn master_system_keeps_its_saves_separate_and_uses_picture_controls() {
    use slot::persist::{read_sav, write_sav};
    use slot::video_mode::{video_mode_for_platform, VideoMode};
    use slot_input::RawEvent;
    let d = common::tmp_root_with_carts(&[]);
    std::fs::write(d.path().join("Games/SMS/Sonic.sms"), [0u8; 0x8000]).unwrap();
    write_sav(d.path(), Platform::Md, "Sonic", b"md save").unwrap();
    write_sav(d.path(), Platform::Sms, "Sonic", b"sms save").unwrap();
    assert_eq!(
        read_sav(d.path(), Platform::Md, "Sonic").unwrap(),
        b"md save"
    );
    assert_eq!(
        read_sav(d.path(), Platform::Sms, "Sonic").unwrap(),
        b"sms save"
    );
    let mut s = common::session_playing(d.path());
    assert_eq!(s.app().canvas(), Platform::Sms.canvas());
    let colour = read_slot_state(d.path()).colour_correction_gb;
    for (t, e) in [
        RawEvent::Down(Btn::Select),
        RawEvent::Down(Btn::Y),
        RawEvent::Up(Btn::Y),
        RawEvent::Up(Btn::Select),
        RawEvent::Down(Btn::L1),
        RawEvent::Up(Btn::L1),
    ]
    .into_iter()
    .enumerate()
    {
        s.feed([e], 100 + 20 * t as u64);
    }
    assert_eq!(read_slot_state(d.path()).colour_correction_gb, colour);
    assert_eq!(
        video_mode_for_platform(d.path(), "Sonic", Platform::Sms),
        VideoMode::Stretch
    );
}
