mod common;
use slot::app::{App, Phase};
use slot_input::Btn;
use slot_store::{write_slot_state, Platform, SlotState};

fn root() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    slot::root::ensure(d.path());
    d
}
#[test]
fn png_carts_and_splore_join_the_shelf_and_cross_system_search() {
    let d = root();
    std::fs::write(d.path().join("Games/PICO8/Adventure.p8.png"), b"fixture").unwrap();
    std::fs::write(
        d.path().join("Games/PICO8/Adventure.p8"),
        b"duplicate plain cart",
    )
    .unwrap();
    std::fs::write(d.path().join("Games/GBA/Adventure.gba"), b"fixture").unwrap();
    let carts = slot_store::scan(d.path()).unwrap();
    assert_eq!(carts.len(), 3);
    let pico = carts
        .iter()
        .find(|c| c.platform == Platform::Pico8 && c.stem == "Adventure")
        .unwrap();
    assert!(pico.rom.ends_with("Adventure.p8.png"));
    assert!(pico.label.is_none());
    assert!(carts.iter().any(|c| c.code == "SLOT_SPLORE"));
    let mut app = App::new(carts);
    app.search_games("Adventure");
    assert_eq!(app.game_search_text(), Some(("Adventure", 2)));
    app.search_games("Splore");
    assert_eq!(app.game_search_text(), Some(("Splore", 1)));
}
#[test]
fn png_face_shows_the_entire_cart_and_leaves_game_bytes_untouched() {
    let d = root();
    let rom = d.path().join("Games/PICO8/Art.p8.png");
    let mut pixels = vec![0; 160 * 205 * 4];
    for p in pixels.chunks_exact_mut(4) {
        p.copy_from_slice(&[15, 30, 45, 252]);
    }
    pixels[..4].copy_from_slice(&[200, 0, 0, 253]);
    let n = pixels.len();
    pixels[n - 4..].copy_from_slice(&[0, 0, 220, 254]);
    {
        let mut enc = png::Encoder::new(std::fs::File::create(&rom).unwrap(), 160, 205);
        enc.set_color(png::ColorType::Rgba);
        enc.write_header()
            .unwrap()
            .write_image_data(&pixels)
            .unwrap();
    }
    let before = std::fs::read(&rom).unwrap();
    let carts = slot_store::scan(d.path()).unwrap();
    let cart = carts.iter().find(|c| c.stem == "Art").unwrap();
    let face = slot_ui::cart_face(cart);
    assert_eq!((face.w, face.h), (320, 410));
    assert_eq!(&face.rgba[..4], &[200, 0, 0, 255]);
    assert_eq!(&face.rgba[face.rgba.len() - 4..], &[0, 0, 220, 255]);
    assert_eq!(before, std::fs::read(&rom).unwrap());
}
#[test]
fn returning_from_native_game_stays_on_its_shelf_without_autolaunch() {
    let d = root();
    std::fs::write(
        d.path().join("Games/PICO8/Adventure.p8"),
        b"pico-8 cartridge",
    )
    .unwrap();
    write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            cart: Some("Adventure".into()),
            cart_platform: Some(Platform::Pico8),
            ..Default::default()
        },
    )
    .unwrap();
    let app = App::boot(d.path());
    assert!(matches!(app.phase(), Phase::Shelf));
    assert_eq!(app.selected_stem(), Some("Adventure"));
}
#[test]
fn missing_runtime_returns_to_shelf_without_starting_an_emulator() {
    let d = root();
    std::fs::write(
        d.path().join("Games/PICO8/Adventure.p8"),
        b"pico-8 cartridge",
    )
    .unwrap();
    write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mut session = slot::session::Session::boot(d.path().to_owned());
    common::open_shelf(session.app_mut());
    common::press(session.app_mut(), Btn::A);
    for _ in 0..90 {
        session.update(1.0 / 60.0);
    }
    assert!(matches!(session.app().phase(), Phase::Shelf));
    assert!(!session.has_core());
    assert!(!session.native_ready());
}
#[test]
fn insertion_prepares_a_native_launch_and_keeps_the_cart_selected_on_return() {
    let d = root();
    std::fs::write(
        d.path().join("Games/PICO8/Adventure.p8"),
        b"pico-8 cartridge",
    )
    .unwrap();
    let mut elf = [0; 64];
    elf[..6].copy_from_slice(b"\x7fELF\x02\x01");
    elf[18] = 183;
    std::fs::write(d.path().join("BIOS/PICO8/pico8_64"), elf).unwrap();
    std::fs::write(d.path().join("BIOS/PICO8/pico8.dat"), b"fixture").unwrap();
    write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mut session = slot::session::Session::boot(d.path().to_owned());
    common::open_shelf(session.app_mut());
    common::press(session.app_mut(), Btn::A);
    for _ in 0..120 {
        session.update(1.0 / 60.0);
    }
    assert!(session.native_ready());
    assert!(!session.has_core());
    assert!(!session.app().records_rewind());
    session.queue_native().unwrap();
    session.app().flush_state();
    assert_eq!(
        std::fs::read_to_string(d.path().join("Config/pico8-launch")).unwrap(),
        d.path().join("Games/PICO8/Adventure.p8").to_str().unwrap()
    );
    let app = App::boot(d.path());
    assert!(matches!(app.phase(), Phase::Shelf));
    assert_eq!(app.selected_stem(), Some("Adventure"));
}
