mod common;

use slot::{app::{App, Phase}, frontend::Frontend};
use slot_input::{Action, Btn, InputSource, RawEvent};
use slot_store::{Cart, Platform};
use std::time::Duration;

#[cfg(unix)]
#[test]
fn constructing_searching_and_leaving_a_library_never_opens_roms() {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let root = tempfile::tempdir().unwrap();
    let carts: Vec<_> = [Platform::Gb, Platform::Gbc, Platform::Nes, Platform::Snes, Platform::Md]
        .into_iter().map(|platform| {
            let rom = root.path().join(format!("{}.{}", platform.dir_name(), platform.extensions()[0]));
            let path = CString::new(rom.as_os_str().as_bytes()).unwrap();
            // Opening a FIFO for reading would wait forever: unlike a timing benchmark,
            // this detects hidden ROM reads even on a fast local filesystem.
            assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
            Cart { platform, stem: platform.dir_name().into(), rom, label: None,
                famicom_label: None, title: String::new(), code: String::new(), shell: None }
        }).collect();
    let (send, recv) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut app = App::new(carts);
        app.show_systems();
        assert_eq!(app.system_grid().unwrap().entries.len(), 5);
        app.search_games("");
        assert_eq!(app.game_search_text(), Some(("", 5)));
        app.apply(Action::GbaDown(Btn::B));
        assert!(app.on_systems());
        send.send(()).unwrap();
    });
    recv.recv_timeout(Duration::from_secs(5)).expect("library preparation opened a ROM");
}

struct Keys(Vec<RawEvent>);
impl InputSource for Keys {
    fn poll(&mut self, _: slot_input::Millis) -> Vec<RawEvent> { std::mem::take(&mut self.0) }
}

#[test]
fn new_labels_do_not_block_boot_input_or_start_a_library_wide_cache_job() {
    let root = common::tmp_root_with_carts(&["Alpha", "Beta"]);
    common::clocked(root.path());
    for name in ["Alpha", "Beta"] {
        let file = std::fs::File::create(root.path().join(format!("Labels/GBA/{name}.png"))).unwrap();
        let mut encoder = png::Encoder::new(file, 32, 32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.write_header().unwrap().write_image_data(&vec![128; 32*32*3]).unwrap();
    }
    let mut frontend = Frontend::boot(Box::new(slot_power::SimPlatform::at(root.path().into())));
    assert!(frontend.app().on_systems());
    frontend.advance(&mut Keys(vec![RawEvent::Down(Btn::A), RawEvent::Up(Btn::A)]));
    assert!(!frontend.app().on_systems(), "uncached labels swallowed the open-shelf input");
    assert!(matches!(frontend.app().phase(), Phase::Shelf));
    assert!(!root.path().join("System/Cache/Labels2").exists(), "startup eagerly cached artwork");
}

#[test]
fn prepared_shadow_shapes_follow_same_named_games_through_search() {
    use slot_ui::{MdShell, ShelfBacking, TexId, Draw};
    let root = common::tmp_root_with_carts(&["Sonic"]);
    std::fs::write(root.path().join("Games/MD/Sonic.md"), b"not read").unwrap();
    let mut app = App::new(slot_store::scan(root.path()).unwrap());
    let japan = TexId::from_raw(901);
    app.set_md_cart_shadows(TexId::from_raw(900), japan);
    app.set_prepared_cart_face(Platform::Md.index(), 0, TexId::from_raw(902), ShelfBacking::Md(MdShell::Japan));
    app.search_games("Sonic");
    let mut draws = Vec::new();
    app.draw(&mut draws);
    assert!(draws.iter().any(|d| matches!(d, Draw::Tex { tex, .. } if *tex == japan)),
        "search lost the worker's resolved cartridge shape");
    app.apply(Action::GbaDown(Btn::B));
    app.search_games("Sonic");
    draws.clear(); app.draw(&mut draws);
    assert!(draws.iter().any(|d| matches!(d, Draw::Tex { tex, .. } if *tex == japan)));
}
