mod common;
use slot::app::{App, Phase};
use slot_input::Btn;
use slot_store::{read_slot_state, write_slot_state, Core, Platform, Shader, SlotState};
use slot_ui::{QuickRow, QuickValue};
#[test]
fn n64_and_pocket_join_search_and_keep_settings_and_saves_separate() {
    let d = common::tmp_root_with_carts(&[]);
    for (dir, ext) in [("N64", "z64"), ("NGP", "ngc")] {
        std::fs::write(
            d.path().join(format!("Games/{dir}/Same Game.{ext}")),
            [0; 256],
        )
        .unwrap();
    }
    let mut a = App::new(slot_store::scan(d.path()).unwrap());
    a.search_games("Same Game");
    assert_eq!(a.game_search_text(), Some(("Same Game", 2)));
    let p: std::collections::HashSet<_> = a.carts().map(|c| c.platform).collect();
    assert!(p.contains(&Platform::N64));
    assert!(p.contains(&Platform::Ngp));
    for (platform, core) in [
        (Platform::N64, Core::ParallelN64),
        (Platform::Ngp, Core::BeetleNgp),
    ] {
        assert_eq!(Core::default_for(platform), core);
        assert!(core.runs(platform));
    }
    for (platform, bytes) in [(Platform::N64, b"n64 save"), (Platform::Ngp, b"ngp save")] {
        slot::persist::write_sav(d.path(), platform, "Same Game", bytes).unwrap();
        assert_eq!(
            slot::persist::read_sav(d.path(), platform, "Same Game").unwrap(),
            bytes
        );
    }
}
#[test]
fn new_screen_pages_are_independent_and_persist() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta"]);
    write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    let (mut a, _) = common::app_booting_at(d.path(), common::CLOCK_IS_SET);
    assert!(matches!(a.phase(), Phase::Shelf));
    for (row, overlay, parent) in [
        (
            QuickRow::N64Shader,
            QuickRow::N64Overlay,
            QuickRow::ScreenN64,
        ),
        (
            QuickRow::NgpShader,
            QuickRow::NgpOverlay,
            QuickRow::ScreenNgp,
        ),
    ] {
        common::open_at(&mut a, row);
        assert_eq!(a.quick_value(row), Some(QuickValue::Off));
        common::press(&mut a, Btn::Right);
        assert_eq!(a.quick_value(row), Some(QuickValue::Grid));
        common::press(&mut a, Btn::Down);
        assert_eq!(a.quick_menu(), Some(overlay));
        common::press(&mut a, Btn::Down);
        assert_eq!(a.quick_menu(), Some(row));
        common::press(&mut a, Btn::B);
        assert_eq!(a.quick_menu(), Some(parent));
        common::press(&mut a, Btn::B);
        common::press(&mut a, Btn::B);
    }
    let s = read_slot_state(d.path());
    assert_eq!(s.shader_n64, Shader::Grid);
    assert_eq!(s.shader_ngp, Shader::Grid);
    assert_eq!(s.shader_sms, Shader::Off);
}
#[test]
fn n64_keeps_shoulders_and_does_not_record_large_rewind_states() {
    let mut a = App::new(Vec::new());
    a.set_platform(Platform::N64);
    assert!(!a.records_rewind());
    assert_eq!(a.canvas(), (320, 240));
    a.set_platform(Platform::Ngp);
    assert_eq!(a.canvas(), (160, 152));
}
