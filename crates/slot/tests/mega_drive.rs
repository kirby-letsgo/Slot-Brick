//! The Mega Drive shelf, its carts and its Screen page, as the app sees them.
mod common;

use common::{open_at, press, rows_of, saved_shell, session_playing};
use slot::app::{App, Phase};
use slot::video_mode::{video_mode_for_platform, VideoMode};
use slot_input::{Btn, RawEvent};
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
    let (mut app, _) = common::app_booting_at(d.path(), common::CLOCK_IS_SET);
    common::open_shelf(&mut app);
    assert!(matches!(app.phase(), Phase::Shelf), "{:?}", app.phase());
    (d, app)
}

#[test]
#[cfg_attr(not(feature = "brick"), ignore = "SELECT + X is a Brick gesture")]
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

#[test]
fn the_mega_drive_shader_is_set_from_its_own_page() {
    let (d, mut a) = booted_on_shelf(common::tmp_root_with_carts(&["Emerald", "Fusion"]));
    open_at(&mut a, QuickRow::MdShader);
    assert_eq!(a.quick_value(QuickRow::MdShader), Some(QuickValue::Off));
    press(&mut a, Btn::Right);
    assert_eq!(read_slot_state(d.path()).shader_md, Shader::Grid);
    assert_eq!(
        read_slot_state(d.path()).shader_snes,
        Shader::Off,
        "the SNES took the Mega Drive one"
    );
    assert_eq!(a.quick_value(QuickRow::MdShader), Some(QuickValue::Grid));
}

#[test]
fn the_mega_drive_page_has_its_shader_and_overlay_and_wraps_between_them() {
    let (_d, mut a) = booted_on_shelf(common::tmp_root_with_carts(&["Emerald", "Fusion"]));
    open_at(&mut a, QuickRow::MdShader);
    press(&mut a, Btn::Down);
    assert_eq!(a.quick_menu(), Some(QuickRow::MdOverlay));
    press(&mut a, Btn::Down);
    assert_eq!(a.quick_menu(), Some(QuickRow::MdShader));
    press(&mut a, Btn::B);
    assert_eq!(a.quick_menu(), Some(QuickRow::ScreenMd));
}

#[test]
fn the_mega_drive_overlay_row_steps_through_its_own_folder_and_saves_it() {
    let d = common::tmp_root_with_carts(&["Emerald", "Fusion"]);
    for (platform, f) in [(Platform::Md, "CRT.png"), (Platform::Snes, "Other.png")] {
        let dir = slot_store::overlays_dir(d.path(), platform);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(f), b"x").unwrap();
    }
    let (d, mut a) = booted_on_shelf(d);
    open_at(&mut a, QuickRow::MdOverlay);
    assert_eq!(a.overlay_text(Platform::Md), "Off");
    press(&mut a, Btn::Right);
    assert_eq!(
        read_slot_state(d.path()).overlay(Platform::Md),
        Some("CRT.png")
    );
    assert_eq!(
        read_slot_state(d.path()).overlay(Platform::Snes),
        None,
        "the SNES took the Mega Drive one"
    );
    assert_eq!(a.overlay_text(Platform::Md), "CRT");
    press(&mut a, Btn::Right);
    assert_eq!(read_slot_state(d.path()).overlay(Platform::Md), None);
}

/// The shape the shelf draws the cart in: its face and its shadow both go by this.
fn shape_on_shelf(app: &App, stem: &str) -> Option<slot_ui::MdShell> {
    app.carts()
        .find(|c| c.stem == stem)
        .and_then(slot_ui::md_shell_of)
}

#[test]
fn a_mega_drive_cart_can_swap_to_the_japanese_shape_and_it_is_saved() {
    use slot_ui::CartRow::*;
    let d = common::tmp_root_with_carts(&[]);
    for stem in ["Sonic (USA)", "Zzz (USA)"] {
        std::fs::write(d.path().join(format!("Games/MD/{stem}.md")), [0u8; 0x200]).unwrap();
    }
    let (d, mut app) = booted_on_shelf(d);
    assert_eq!(app.selected_stem(), Some("Sonic (USA)"));
    press(&mut app, Btn::Start);
    assert_eq!(
        rows_of(&mut app),
        vec![Shape, Reset],
        "a Mega Drive cart offered a colour"
    );
    press(&mut app, Btn::Left);
    let japan = app.cart_menu().unwrap().1.expect("a shape is a change");
    assert_eq!(japan.outline, slot_store::Outline::Rounded);
    assert_eq!(
        shape_on_shelf(&app, "Sonic (USA)"),
        Some(slot_ui::MdShell::Japan),
        "the cart on the shelf kept its Western shape"
    );
    press(&mut app, Btn::A);
    let saved = saved_shell(&d, Platform::Md, "Sonic (USA)").expect("A saved nothing");
    assert_eq!(
        slot_store::ini::value(d.path(), slot_store::LABELS_SHELL_FILE, "Sonic (USA)"),
        None,
        "kept under its bare name, shared with other systems"
    );
    assert!(saved.starts_with("rounded "), "{saved}");
}

#[test]
fn a_japanese_mega_drive_cart_can_go_on_the_western_shape_and_it_is_saved() {
    let d = common::tmp_root_with_carts(&[]);
    for stem in ["Puyo Puyo (Japan)", "Zzz (Japan)"] {
        std::fs::write(d.path().join(format!("Games/MD/{stem}.md")), [0u8; 0x200]).unwrap();
    }
    let (d, mut app) = booted_on_shelf(d);
    assert_eq!(app.selected_stem(), Some("Puyo Puyo (Japan)"));
    assert_eq!(
        shape_on_shelf(&app, "Puyo Puyo (Japan)"),
        Some(slot_ui::MdShell::Japan)
    );
    press(&mut app, Btn::Start);
    press(&mut app, Btn::Right);
    let west = app.cart_menu().unwrap().1.expect("a shape is a change");
    assert_eq!(west.outline, slot_store::Outline::Notched);
    assert_eq!(
        shape_on_shelf(&app, "Puyo Puyo (Japan)"),
        Some(slot_ui::MdShell::West)
    );
    press(&mut app, Btn::A);
    let saved = saved_shell(&d, Platform::Md, "Puyo Puyo (Japan)").expect("A saved nothing");
    assert!(saved.starts_with("notched "), "{saved}");
}
