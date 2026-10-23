mod common;

use slot_retro::RetroCore;
use slot_store::Core;

/// The simulator's build of the core when there is one (it runs on this PC: the sim's Docker
/// image), else the Brick's.
fn dylib() -> std::path::PathBuf {
    let name = slot::core::dylib_name(Core::GenesisPlusGx);
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let sim = root.join("target-sim/cores").join(&name);
    if sim.exists() {
        sim
    } else {
        root.join("vendor").join(name)
    }
}

/// A 32 KiB ROM whose program sets the backdrop red in the 320-wide mode, turns the display
/// on and loops. No TMSS write: the core runs without the BIOS.
fn red_screen_rom() -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0..4].copy_from_slice(&0x00ff_fe00u32.to_be_bytes()); // stack
    rom[4..8].copy_from_slice(&0x0000_0200u32.to_be_bytes()); // entry
    rom[0x100..0x110].copy_from_slice(b"SEGA MEGA DRIVE ");
    rom[0x190] = b'J'; // a 3-button pad
    rom[0x1a4..0x1a8].copy_from_slice(&0x0000_7fffu32.to_be_bytes());
    rom[0x1f0..0x1f3].copy_from_slice(b"JUE");
    // Registers above 10 are ignored until mode 5 is on, and colours keep only their top bit
    // until register 0 has its palette-select bit.
    #[rustfmt::skip]
    let program: [u16; 24] = [
        0x41f9, 0x00c0, 0x0004, // lea $c00004,a0: VDP control
        0x43f9, 0x00c0, 0x0000, // lea $c00000,a1: VDP data
        0x30bc, 0x8004,         // move.w #$8004,(a0): full palette
        0x30bc, 0x8104,         // mode 5, display still off
        0x30bc, 0x8c81,         // 320 wide
        0x30bc, 0x8f02,         // auto-increment 2
        0x30bc, 0x8700,         // the backdrop is colour 0
        0x20bc, 0xc000, 0x0000, // move.l #$c0000000,(a0): write colour 0 ...
        0x32bc, 0x000e,         // ... move.w #$000e,(a1): red
        0x30bc, 0x8144,         // display on
        0x60fe,                 // bra.s to itself
    ];
    for (i, w) in program.iter().enumerate() {
        rom[0x200 + 2 * i..0x202 + 2 * i].copy_from_slice(&w.to_be_bytes());
    }
    rom
}

fn load(d: &std::path::Path) -> Option<Box<dyn RetroCore>> {
    let path = dylib();
    if !path.exists() {
        eprintln!("no genesis_plus_gx dylib on this host, skipping");
        return None;
    }
    let rom = d.join("red.md");
    std::fs::write(&rom, red_screen_rom()).unwrap();
    let mut core = slot::core::open_core_for(d, Core::GenesisPlusGx, "auto", false, &[path]);
    core.load(&rom).expect("genesis_plus_gx refused the cart");
    Some(core)
}

#[test]
fn a_mega_drive_game_draws_its_picture_centred_on_its_canvas() {
    let _g = common::core_lock();
    let d = tempfile::tempdir().unwrap();
    let Some(mut core) = load(d.path()) else {
        return;
    };
    for _ in 0..10 {
        core.run_frame(slot_retro::ButtonMask(0));
    }
    let (w, h) = core.canvas();
    assert_eq!((w, h), slot_store::Platform::Md.canvas());
    let video = core.video_xrgb8888();
    let px = |x: u32, y: u32| {
        let i = ((y * w + x) * 4) as usize;
        [video[i + 2], video[i + 1], video[i]]
    };
    let red = |p: [u8; 3]| p[0] > 0xc0 && p[1] < 0x40 && p[2] < 0x40;
    assert!(
        red(px(160, 120)),
        "the picture is not red: {:?}",
        px(160, 120)
    );
    assert!(
        red(px(2, 120)) && red(px(317, 120)),
        "the picture is not 320 wide"
    );
    assert_eq!(
        px(160, 3),
        [0, 0, 0],
        "a 224-line picture is not centred in the 240-line canvas"
    );
    let state = core.serialize().expect("no save state");
    core.run_frame(slot_retro::ButtonMask(0));
    core.unserialize(&state)
        .expect("its own save state was refused");
}

#[test]
fn every_option_slot_sets_on_genesis_plus_gx_is_one_it_declares() {
    let path = dylib();
    if !path.exists() {
        eprintln!("no genesis_plus_gx dylib on this host, skipping");
        return;
    }
    let _g = common::core_lock();
    let mut core = slot_retro::LibretroCore::open(&path).expect("open genesis_plus_gx");
    slot::core::apply_core_options(&mut core, Core::GenesisPlusGx, "auto", false, false, None);
    let declared = core.declared_options();
    assert!(
        !declared.is_empty(),
        "the core declared no options, so this would pass on anything"
    );
    let set = core.options();
    assert!(
        set.iter()
            .any(|(k, v)| k == "genesis_plus_gx_bios" && v == "disabled"),
        "{set:?}"
    );
    for (key, value) in set {
        let values = declared
            .get(&key)
            .unwrap_or_else(|| panic!("genesis_plus_gx declares no option {key:?}"));
        assert!(
            values.contains(&value),
            "genesis_plus_gx declares {key:?} as {values:?}, not {value:?}"
        );
    }
}
