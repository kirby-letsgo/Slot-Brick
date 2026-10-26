use slot_store::Platform;

#[test]
fn every_platform_has_a_directory() {
    let names: Vec<&str> = Platform::ALL.iter().map(|p| p.dir_name()).collect();
    assert_eq!(
        names,
        vec!["GBA", "GB", "GBC", "NES", "SNES", "MD", "PS1", "GG", "SMS", "N64", "NGP", "PICO8", "32X", "SEGACD", "PCE", "PCECD", "NEOGEO", "WS", "LYNX", "PORTS"]
    );
}

#[test]
fn every_platform_is_a_shelf_of_its_own() {
    assert_eq!(
        Platform::ALL,
        [
            Platform::Gba,
            Platform::Gb,
            Platform::Gbc,
            Platform::Nes,
            Platform::Snes,
            Platform::Md,
            Platform::Ps1,
            Platform::Gg,
            Platform::Sms,
            Platform::N64,
            Platform::Ngp,
            Platform::Pico8,
            Platform::S32x,
            Platform::SegaCd,
            Platform::Pce,
            Platform::PceCd,
            Platform::NeoGeo,
            Platform::Ws,
            Platform::Lynx,
            Platform::Ports
        ]
    );
    for (i, p) in Platform::ALL.iter().enumerate() {
        assert!(
            !Platform::ALL[..i].contains(p),
            "{p:?} is named twice in the ring, so two shelves would hold one platform"
        );
    }
}

#[test]
fn each_platform_takes_only_its_own_extensions() {
    assert!(Platform::Gba.accepts(std::path::Path::new("Metroid Fusion.gba")));
    assert!(!Platform::Gba.accepts(std::path::Path::new("Tetris.gb")));
    assert!(Platform::Gb.accepts(std::path::Path::new("Tetris.gb")));
    assert!(Platform::Gb.accepts(std::path::Path::new("Tetris.gbc")));
    assert!(!Platform::Gb.accepts(std::path::Path::new("Metroid Fusion.gba")));
    assert!(Platform::Gba.accepts(std::path::Path::new("Shrek.GBA")));
    assert!(Platform::Nes.accepts(std::path::Path::new("Super Mario Bros. (World).nes")));
    assert!(!Platform::Nes.accepts(std::path::Path::new("Tetris.gb")));
    assert!(!Platform::Gba.accepts(std::path::Path::new("Zelda.NES")));
    for disc in ["Ridge Racer.cue", "Ridge Racer.chd", "Ridge Racer.PBP", "Final Fantasy VII.m3u"] {
        assert!(Platform::Ps1.accepts(std::path::Path::new(disc)), "{disc}");
    }
    assert!(!Platform::Ps1.accepts(std::path::Path::new("Ridge Racer.sfc")));
}

#[test]
fn a_nes_picture_fits_its_canvas_and_the_game_boys_keep_theirs() {
    assert_eq!(Platform::Nes.picture(), (256, 224));
    assert_eq!(Platform::Nes.canvas(), (256, 240));
    for p in [Platform::Gba, Platform::Gb, Platform::Gbc] {
        assert_eq!(p.canvas(), (240, 160), "{p:?}");
    }
    // Every PlayStation frame is stretched over a 4:3 canvas, its picture all of it.
    assert_eq!(Platform::Ps1.picture(), Platform::Ps1.canvas());
}

#[test]
fn a_mega_drive_takes_its_four_extensions_and_a_320_wide_picture() {
    for rom in ["Sonic.md", "Sonic.gen", "Sonic.SMD", "Sonic.bin"] {
        assert!(Platform::Md.accepts(std::path::Path::new(rom)), "{rom}");
    }
    assert!(!Platform::Md.accepts(std::path::Path::new("Sonic.sms")));
    assert_eq!(Platform::Md.name(), "Sega Mega Drive");
    assert_eq!(Platform::Md.picture(), (320, 224));
    assert_eq!(Platform::Md.canvas(), (320, 240));
    for (i, p) in Platform::ALL.iter().enumerate() {
        assert_eq!(p.index(), i, "{p:?}");
    }
}

#[test]
fn master_system_accepts_sms_and_uses_a_four_by_three_canvas() {
    assert!(Platform::Sms.accepts(std::path::Path::new("Sonic.SMS")));
    assert!(!Platform::Sms.accepts(std::path::Path::new("Sonic.gg")));
    assert!(!Platform::Gg.accepts(std::path::Path::new("Sonic.sms")));
    assert_eq!(Platform::Sms.picture(), (256, 192));
    assert_eq!(Platform::Sms.canvas(), (256, 192));
}
