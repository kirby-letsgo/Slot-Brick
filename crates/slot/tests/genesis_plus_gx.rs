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

/// A 32 KiB ROM whose program sets the backdrop red in the 320- or 256-wide mode, turns the display
/// on and loops (`reg12` picks which). No TMSS write: the core runs without the BIOS.
fn red_screen_rom(reg12: u16) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0..4].copy_from_slice(&0x00ff_fe00u32.to_be_bytes()); // stack
    rom[4..8].copy_from_slice(&0x0000_0200u32.to_be_bytes()); // entry
    rom[0x100..0x110].copy_from_slice(b"SEGA MEGA DRIVE ");
    rom[0x190] = b'J'; // a 3-button pad
    rom[0x1a4..0x1a8].copy_from_slice(&0x0000_7fffu32.to_be_bytes());
    rom[0x1f0..0x1f3].copy_from_slice(b"JUE");
    // Registers above 10 are ignored until mode 5 is on, and colours keep only their lowest
    // bit per channel until register 0 has its palette-select bit.
    #[rustfmt::skip]
    let program: [u16; 24] = [
        0x41f9, 0x00c0, 0x0004, // lea $c00004,a0: VDP control
        0x43f9, 0x00c0, 0x0000, // lea $c00000,a1: VDP data
        0x30bc, 0x8004,         // move.w #$8004,(a0): full palette
        0x30bc, 0x8104,         // mode 5, display still off
        0x30bc, reg12,          // 320 wide (0x8c81) or 256 wide (0x8c00)
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

fn load(d: &std::path::Path, reg12: u16) -> Option<Box<dyn RetroCore>> {
    let path = dylib();
    if !path.exists() {
        eprintln!("no genesis_plus_gx dylib on this host, skipping");
        return None;
    }
    let rom = d.join("red.md");
    std::fs::write(&rom, red_screen_rom(reg12)).unwrap();
    let mut core = slot::core::open_core_for(d, Core::GenesisPlusGx, "auto", false, &[path]);
    core.load(&rom).expect("genesis_plus_gx refused the cart");
    let av = core.av_info();
    assert!(
        av.sample_rate == 44_100.0 && (av.fps - 59.92).abs() < 0.05,
        "{av:?}: a mock core stood in for genesis_plus_gx, or the JUE cart went PAL"
    );
    Some(core)
}

fn the_picture_fills_the_canvas_width(reg12: u16) {
    let _g = common::core_lock();
    let d = tempfile::tempdir().unwrap();
    let Some(mut core) = load(d.path(), reg12) else {
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
        "the picture does not span the canvas"
    );
    assert_eq!(
        px(160, 3),
        [0, 0, 0],
        "a 224-line picture is not centred in the 240-line canvas"
    );
    assert_eq!(
        px(160, 236),
        [0, 0, 0],
        "the band below the picture is not black"
    );
    assert!(
        red(px(160, 8)) && red(px(160, 231)),
        "the picture is not 224 lines from line 8"
    );
    let state = core.serialize().expect("no save state");
    core.run_frame(slot_retro::ButtonMask(0));
    core.unserialize(&state)
        .expect("its own save state was refused");
}

#[test]
fn a_mega_drive_game_draws_its_picture_centred_on_its_canvas() {
    the_picture_fills_the_canvas_width(0x8c81);
}

#[test]
fn a_256_wide_mega_drive_game_fills_the_canvas_width_too() {
    the_picture_fills_the_canvas_width(0x8c00);
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

#[test]
fn a_game_gear_rom_draws_160_by_144_and_round_trips_a_save_state() {
    let path = dylib();
    if !path.exists() {
        eprintln!("no genesis_plus_gx dylib on this host, skipping");
        return;
    }
    let _g = common::core_lock();
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir(d.path().join("System")).unwrap();
    std::fs::copy(
        path,
        d.path()
            .join("System")
            .join(slot::core::dylib_name(Core::GenesisPlusGx)),
    )
    .unwrap();
    // A homebrew Z80 program: mode 4, display blanked, backdrop colour 16 red.
    let mut rom = vec![0u8; 0x8000];
    let mut program = vec![0xf3]; // DI
    for (port, value) in [
        (0xbf, 4),
        (0xbf, 0x80),
        (0xbf, 0),
        (0xbf, 0x81),
        (0xbf, 0),
        (0xbf, 0x87),
        (0xbf, 32),
        (0xbf, 0xc0),
        (0xbe, 15),
        (0xbe, 0),
    ] {
        program.extend_from_slice(&[0x3e, value, 0xd3, port]); // LD A,n; OUT (port),A
    }
    let poll = program.len() as u16;
    // Poll buttons 1/2 and Start into work RAM so the host can verify the pad mapping.
    program.extend_from_slice(&[
        0xdb, 0xdc, 0x32, 0x00, 0xc0, 0xdb, 0x00, 0x32, 0x01, 0xc0, 0xc3,
    ]);
    program.extend_from_slice(&poll.to_le_bytes());
    rom[..program.len()].copy_from_slice(&program);
    rom[0x7ff0..0x7ff8].copy_from_slice(b"TMR SEGA");
    rom[0x7fff] = 0x6c; // export Game Gear, 32 KiB
    let cart = d.path().join("red.gg");
    std::fs::write(&cart, rom).unwrap();
    let opened = slot::core::open_game_gear(d.path());
    assert!(opened.named, "a mock replaced Genesis Plus GX");
    let mut core = opened.core;
    core.load(&cart).unwrap();
    for _ in 0..10 {
        core.run_frame(slot_retro::ButtonMask(0));
    }
    assert_eq!(core.canvas(), (240, 160));
    let video = core.video_xrgb8888();
    for y in 0..160 {
        for x in 0..240 {
            let i = (y * 240 + x) * 4;
            let rgb = [video[i + 2], video[i + 1], video[i]];
            if (40..200).contains(&x) && (8..152).contains(&y) {
                assert!(
                    rgb[0] > 0xc0 && rgb[1] < 0x40 && rgb[2] < 0x40,
                    "({x},{y}): {rgb:?}"
                );
            } else {
                assert_eq!(rgb, [0, 0, 0], "outside picture at ({x},{y})");
            }
        }
    }
    let buttons = |core: &mut dyn RetroCore| {
        let (ram, len) = core.memory_region(2).expect("no system RAM");
        assert!(len >= 2);
        // The core owns this buffer; no frame runs while these two bytes are read.
        unsafe { [*ram & 0x30, *ram.add(1) & 0x80] }
    };
    assert_eq!(buttons(core.as_mut()), [0x30, 0x80]);
    for (pressed, expected) in [
        (slot_retro::ButtonMask::B, [0x20, 0x80]),
        (slot_retro::ButtonMask::A, [0x10, 0x80]),
        (slot_retro::ButtonMask::START, [0x30, 0]),
    ] {
        core.run_frame(slot_retro::ButtonMask(pressed));
        assert_eq!(buttons(core.as_mut()), expected);
    }
    let state = core.serialize().unwrap();
    core.run_frame(slot_retro::ButtonMask(0));
    assert_eq!(buttons(core.as_mut()), [0x30, 0x80]);
    core.unserialize(&state).unwrap();
    assert_eq!(buttons(core.as_mut()), [0x30, 0]);
}

/// A homebrew SMS cart checks its complete TV picture, pad buttons and console Pause NMI.
fn master_system_picture_and_controls(mode: u8) {
    let path = dylib();
    if !path.exists() {
        eprintln!("no genesis_plus_gx dylib on this host, skipping");
        return;
    }
    let _g = common::core_lock();
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir(d.path().join("System")).unwrap();
    std::fs::copy(
        path,
        d.path()
            .join("System")
            .join(slot::core::dylib_name(Core::GenesisPlusGx)),
    )
    .unwrap();
    let mut rom = vec![0u8; 0x8000];
    // Reset jumps past the NMI vector. The Pause NMI increments RAM byte 1, preserving AF.
    rom[..3].copy_from_slice(&[0xc3, 0x00, 0x01]);
    rom[0x66..0x71].copy_from_slice(&[0xf5, 0x3a, 1, 0xc0, 0x3c, 0x32, 1, 0xc0, 0xf1, 0xed, 0x45]);
    let mut program = vec![0xf3, 0x31, 0xf0, 0xdf, 0xaf, 0x32, 1, 0xc0]; // DI; stack; clear NMI count
    for (port, value) in [
        (0xbf, 6),
        (0xbf, 0x80),
        (0xbf, mode),
        (0xbf, 0x81),
        (0xbf, 0),
        (0xbf, 0x87),
        (0xbf, 16),
        (0xbf, 0xc0),
        (0xbe, 3),
    ] {
        program.extend_from_slice(&[0x3e, value, 0xd3, port]);
    }
    let poll = 0x100 + program.len() as u16;
    program.extend_from_slice(&[0xdb, 0xdc, 0x32, 0, 0xc0, 0xc3]);
    program.extend_from_slice(&poll.to_le_bytes());
    rom[0x100..0x100 + program.len()].copy_from_slice(&program);
    rom[0x7ff0..0x7ff8].copy_from_slice(b"TMR SEGA");
    rom[0x7fff] = 0x4c; // export SMS, 32 KiB
    let cart = d.path().join("red.sms");
    std::fs::write(&cart, rom).unwrap();
    let opened = slot::core::open_master_system(d.path());
    assert!(opened.named, "a mock replaced Genesis Plus GX");
    let mut core = opened.core;
    core.load(&cart).unwrap();
    for _ in 0..10 {
        core.run_frame(slot_retro::ButtonMask(0));
    }
    assert_eq!(core.canvas(), (256, 192));
    assert!(
        core.video_xrgb8888()
            .chunks_exact(4)
            .all(|p| p[2] > 0xc0 && p[1] < 0x40 && p[0] < 0x40),
        "the full picture is not red in mode {mode}"
    );
    let read = |core: &mut dyn RetroCore| {
        let (ram, len) = core.memory_region(2).unwrap();
        assert!(len >= 2);
        // The core owns the buffer, and no frame runs during these reads.
        unsafe { [*ram & 0x30, *ram.add(1)] }
    };
    assert_eq!(read(core.as_mut()), [0x30, 0]);
    for (button, expected) in [
        (slot_retro::ButtonMask::B, 0x20),
        (slot_retro::ButtonMask::A, 0x10),
    ] {
        core.run_frame(slot_retro::ButtonMask(button));
        assert_eq!(read(core.as_mut()), [expected, 0]);
    }
    core.run_frame(slot_retro::ButtonMask(slot_retro::ButtonMask::START));
    core.run_frame(slot_retro::ButtonMask(slot_retro::ButtonMask::START));
    assert_eq!(
        read(core.as_mut()),
        [0x30, 1],
        "holding Start must pause once"
    );
    let state = core.serialize().unwrap();
    core.run_frame(slot_retro::ButtonMask(0));
    core.run_frame(slot_retro::ButtonMask(slot_retro::ButtonMask::START));
    assert_eq!(read(core.as_mut()), [0x30, 2]);
    core.unserialize(&state).unwrap();
    assert_eq!(read(core.as_mut()), [0x30, 1]);
}

#[test]
fn master_system_192_line_picture_buttons_pause_and_state_restore() {
    master_system_picture_and_controls(0);
}

#[test]
fn master_system_224_and_240_line_modes_fit_the_tv_picture() {
    master_system_picture_and_controls(0x10);
    master_system_picture_and_controls(0x08);
}
