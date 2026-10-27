mod common;

use slot::app::{App, Phase};
use slot_input::{Action, Btn};
use slot_store::{GameSettings, GameSettingsStore, Platform, Shader};
use slot_ui::PauseChoice;

fn tap(app:&mut App,btn:Btn) {
    app.apply(Action::GbaDown(btn));
    app.apply(Action::GbaUp(btn));
}

fn fixture()->(tempfile::TempDir,App) {
    let dir=common::tmp_root_with_carts(&["Alpha", "Beta"]);
    std::fs::write(dir.path().join("Games/GG/Alpha.gg"), [0u8;32]).unwrap();
    common::clocked(dir.path());
    let (mut app,_) = common::app_booting_at(dir.path(), 1_800_000_000);
    common::open_shelf(&mut app);
    (dir,app)
}

fn open_gba_settings(app:&mut App) {
    tap(app,Btn::Start); // GBA opens the core chooser.
    tap(app,Btn::X); // X opens emulator settings without changing the selected core.
    assert!(app.game_settings_view().is_some(),"GBA core picker lacks settings shortcut");
}

#[test]
fn game_fps_and_overrides_are_isolated_from_other_games_and_systems() {
    let (dir,mut app)=fixture();
    assert_eq!(app.selected_stem(),Some("Alpha"));
    open_gba_settings(&mut app);
    let view=app.game_settings_view().unwrap();
    assert_eq!(view.rows[0].0,"SHOW FPS");
    assert_eq!(view.rows[0].1,"OFF");
    tap(&mut app,Btn::A);
    assert_eq!(app.game_settings_view().unwrap().rows[0].1,"ON");
    tap(&mut app,Btn::Down);
    tap(&mut app,Btn::Right); // Explicit shader override rather than system default.
    assert_eq!(app.game_settings_view().unwrap().rows[1].1,"OFF");
    tap(&mut app,Btn::Down);
    tap(&mut app,Btn::Right); // Explicit 2x fast-forward for Alpha only.
    assert_eq!(app.game_settings_view().unwrap().rows[2].1,"2X");
    tap(&mut app,Btn::B);
    app.settle_saves();
    let store=GameSettingsStore::read(dir.path());
    let alpha=store.get(Platform::Gba,"Alpha");
    assert_eq!(alpha.show_fps,true);
    assert_eq!(alpha.shader,Some(Shader::Off));
    assert_eq!(alpha.ff_speed,Some(2));
    assert_eq!(store.get(Platform::Gba,"Beta"),GameSettings::default());
    assert_eq!(store.get(Platform::Gg,"Alpha"),GameSettings::default());
    tap(&mut app,Btn::B); // The GBA lid closes with an animation.
    for _ in 0..180 { app.update(1.0/60.0); }
    tap(&mut app,Btn::Right); // Another GBA game inherits the system values.
    assert_eq!(app.selected_stem(),Some("Beta"));
    assert_eq!(app.ff_speed(),slot_store::SlotState::default().ff_speed);
    tap(&mut app,Btn::R1); // Game Gear version of Alpha.
    assert_eq!(app.selected_cart().unwrap().platform,Platform::Gg);
    tap(&mut app,Btn::Start); // Non-GBA cartridge options.
    tap(&mut app,Btn::A); // EMULATOR SETTINGS is first on a fixed-shape cart.
    assert!(app.game_settings_view().is_some());
    assert_eq!(app.game_settings_view().unwrap().rows[0].1,"OFF");
    tap(&mut app,Btn::B);
    tap(&mut app,Btn::B);
    // Restart uses the same config file with no global settings modified.
    let restarted=App::boot(dir.path());
    assert_eq!(GameSettingsStore::read(dir.path()).get(Platform::Gba,"Alpha"),alpha);
    assert_eq!(restarted.ff_speed(),slot_store::SlotState::default().ff_speed);
}

#[test]
fn restoring_a_games_defaults_keeps_other_games_settings() {
    let (dir,mut app)=fixture();
    open_gba_settings(&mut app);
    tap(&mut app,Btn::A);
    tap(&mut app,Btn::Up); // Wrap from FPS to Restore.
    assert_eq!(app.game_settings_view().unwrap().rows.last().unwrap().0,"RESTORE GAME DEFAULTS");
    tap(&mut app,Btn::A);
    assert_eq!(app.game_settings_view().unwrap().rows[0].1,"OFF");
    tap(&mut app,Btn::B);
    app.settle_saves();
    assert_eq!(GameSettingsStore::read(dir.path()).get(Platform::Gba,"Alpha"),GameSettings::default());
}

#[test]
fn emulator_settings_from_pause_do_not_resume_or_reset_the_game() {
    let dir=common::tmp_root_with_carts(&["Emerald"]);
    let mut app=common::app_playing_in(dir.path(),"Emerald");
    app.set_game_ready(true);
    tap(&mut app,Btn::B); // no change on gameplay
    app.apply(Action::QuickMenu);
    let index=PauseChoice::ALL.iter().position(|r|*r==PauseChoice::EmulatorSettings).unwrap();
    for _ in 0..index { tap(&mut app,Btn::Down); }
    tap(&mut app,Btn::A);
    assert!(matches!(app.phase(),Phase::Playing{..}));
    assert!(app.game_settings_view().is_some());
    tap(&mut app,Btn::A); // FPS enabled for this game.
    assert_eq!(app.game_settings_view().unwrap().rows[0].1,"ON");
    assert!(!app.fps_overlay_visible(),"pausing must hide the FPS overlay");
    tap(&mut app,Btn::B);
    assert!(app.game_settings_view().is_none());
    assert!(app.pause_menu().is_some(),"B should return to the pause menu");
    tap(&mut app,Btn::B); // Resume.
    assert!(app.pause_menu().is_none());
    assert!(app.fps_overlay_visible());
    app.settle_saves();
    assert!(GameSettingsStore::read(dir.path()).get(Platform::Gba,"Emerald").show_fps);
}
