//! Optional integration smoke tests against locally built native cores. No game is bundled.
//! SLOT_TEST_CORE_DIR must contain both new .so files; SLOT_TEST_N64_ROM is a homebrew ROM.
use slot_retro::ButtonMask;
use slot_store::Core;
use std::path::PathBuf;
fn setup(which: Core) -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    slot::root::ensure(d.path());
    let cores =
        PathBuf::from(std::env::var_os("SLOT_TEST_CORE_DIR").expect("native core directory"));
    let name = slot::core::dylib_name(which);
    std::fs::copy(cores.join(&name), d.path().join("System").join(name)).unwrap();
    d
}
#[test]
#[ignore = "requires native ParaLLEl N64, EGL/GLES2 and a homebrew ROM"]
fn n64_renders_restores_saves_and_reloads_its_graphics_context() {
    let d = setup(Core::ParallelN64);
    let rom = PathBuf::from(std::env::var_os("SLOT_TEST_N64_ROM").expect("homebrew ROM"));
    for _ in 0..2 {
        let opened = slot::core::open_core(d.path(), Core::ParallelN64, "", false, None, None);
        assert!(opened.named);
        let mut core = opened.core;
        if let Ok(renderer) = std::env::var("SLOT_TEST_N64_RENDERER") {
            core.set_option("parallel-n64-gfxplugin", &renderer);
        }
        if let Ok(cpu) = std::env::var("SLOT_TEST_N64_CPU") {
            core.set_option("parallel-n64-cpucore", &cpu);
        }
        core.load(&rom).unwrap();
        assert_eq!(core.canvas(), (320, 240));
        for _ in 0..90 {
            core.run_frame(ButtonMask::default());
            core.take_audio();
        }
        let pixels = core.video_xrgb8888();
        assert!(
            pixels.chunks_exact(4).any(|p| p[..3] != [0, 0, 0]),
            "visible game pixels"
        );
        assert!(
            pixels.chunks_exact(4).any(|p| p[..3] != pixels[..3]),
            "non-uniform image"
        );
        let state = core.serialize().unwrap();
        assert!(state.len() > 1_000_000);
        core.run_frame(ButtonMask::default());
        core.unserialize(&state).unwrap();
        let mut save = core.save_ram().expect("EEPROM/SRAM and controller pak");
        save[0] ^= 0x55;
        core.load_save_ram(&save).unwrap();
        assert_eq!(core.save_ram().unwrap(), save);
    }
}
#[test]
#[ignore = "requires native Beetle NeoPop"]
fn pocket_mono_and_colour_restore_states_and_keep_flash_in_their_own_folder() {
    let d = setup(Core::BeetleNgp);
    for (mode, ext) in [(0, "ngp"), (0x10, "ngc")] {
        let stem = format!("Pocket {ext}");
        // Our own minimal TLCS-900H program: JR true,-2. It runs the display timers.
        let mut rom = vec![0; 0x10000];
        rom[0x1c..0x20].copy_from_slice(&0x200040u32.to_le_bytes());
        rom[0x23] = mode;
        rom[0x24..0x30].copy_from_slice(b"SLOT TEST   ");
        rom[0x40..0x42].copy_from_slice(&[0x68, 0xfe]);
        let path = d.path().join(format!("Games/NGP/{stem}.{ext}"));
        std::fs::write(&path, rom).unwrap();
        // A valid one-byte flash block; the core must load and write it under Saves/NGP.
        let flash = vec![0x53, 0, 1, 0, 17, 0, 0, 0, 0, 1, 0x20, 0, 1, 0, 0, 0, 0x5a];
        let save = d.path().join(format!("Saves/NGP/{stem}.flash"));
        std::fs::write(&save, &flash).unwrap();
        let opened = slot::core::open_core(d.path(), Core::BeetleNgp, "", false, None, None);
        assert!(opened.named);
        let mut core = opened.core;
        core.load(&path).unwrap();
        assert_eq!(core.canvas(), (160, 152));
        for _ in 0..10 {
            core.run_frame(ButtonMask::default());
            core.take_audio();
        }
        let state = core.serialize().unwrap();
        assert!(!state.is_empty());
        core.unserialize(&state).unwrap();
        std::fs::remove_file(&save).unwrap();
        drop(core);
        let written = std::fs::read(&save).expect("core saves flash on unload");
        assert_eq!(&written[..14], &flash[..14]);
        assert_eq!(written.last(), Some(&0x5a));
        assert!(!d.path().join(format!("Saves/{stem}.flash")).exists());
    }
}
