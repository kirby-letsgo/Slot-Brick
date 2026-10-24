//! Where a NES, SNES or Mega Drive ROM was sold, read from the ROM itself: a ripped cart's
//! file name often has no "(Japan)" or "(Europe)" in it.
use std::path::Path;

/// The SNES header's destination code (0x00 Japan, 0x01 North America, 0x02 Europe, ...), from
/// whichever of the LoROM, HiROM and ExHiROM headers checks out. A copier's 512-byte header
/// before the ROM is skipped.
pub fn snes_destination(rom: &Path) -> Option<u8> {
    let len = crate::archive::len(rom).ok()?;
    let skip = if len % 1024 == 512 { 512 } else { 0 };
    let mut fallback = None;
    for at in [0x7fc0u64, 0xffc0, 0x40_ffc0] {
        if skip + at + 0x20 > len {
            continue;
        }
        let mut h = [0u8; 0x20];
        crate::archive::read_at(rom, skip + at, &mut h).ok()?;
        let complement = u16::from_le_bytes([h[0x1c], h[0x1d]]);
        let checksum = u16::from_le_bytes([h[0x1e], h[0x1f]]);
        let destination = h[0x19];
        if destination > 0x14 {
            continue;
        }
        if checksum ^ complement == 0xffff {
            return Some(destination);
        }
        // A hack or a bad dump can carry a stale checksum: a sane map mode will do.
        if fallback.is_none() && h[0x15] & 0xe0 == 0x20 {
            fallback = Some(destination);
        }
    }
    fallback
}

/// Sold in Japan or a PAL country, so on the rounder PAL / Super Famicom cart: None when the
/// header says nothing either way.
pub fn snes_pal_or_japan(rom: &Path) -> Option<bool> {
    // Japan, Europe and its countries (0x02..=0x0a), Australia.
    snes_destination(rom).map(|d| matches!(d, 0x00 | 0x02..=0x0a | 0x11))
}

/// Mappers only Japanese (Famicom) carts used: Konami's VRC chips, Namco 163, Jaleco, Irem,
/// Sunsoft-3 and Taito boards that never came west.
const FAMICOM_ONLY: &[u16] = &[18, 19, 21, 22, 23, 24, 25, 26, 65, 67, 73, 75, 80, 82, 85, 88, 95, 210];

/// The iNES mapper number, None when the file has no iNES header.
pub fn nes_mapper(rom: &Path) -> Option<u16> {
    let mut h = [0u8; 16];
    crate::archive::read_at(rom, 0, &mut h).ok()?;
    if &h[..4] != b"NES\x1a" {
        return None;
    }
    let nes2 = h[7] & 0x0c == 0x08;
    // An old dumper's text in bytes 7..16 ("DiskDude!") is no mapper: only the low nibble counts.
    let high = if nes2 || h[12..].iter().all(|b| *b == 0) { h[7] & 0xf0 } else { 0 };
    let mut mapper = (h[6] >> 4) as u16 | high as u16;
    if nes2 {
        mapper |= ((h[8] & 0x0f) as u16) << 8;
    }
    Some(mapper)
}

/// A board only Famicom carts had. Most Famicom games share boards with NES ones, so this
/// spots only some; the rest go by name or the cart menu.
pub fn nes_famicom_board(rom: &Path) -> bool {
    nes_mapper(rom).is_some_and(|m| FAMICOM_ONLY.contains(&m))
}

/// The first 0x200 bytes of a Mega Drive ROM as the console sees them. A file without "SEGA" at
/// 0x100 that is an odd number of 512-byte blocks is a copier's .smd dump: its header comes off
/// and its first 16 KiB block, odd bytes stored before even ones, is put back in order (as
/// Genesis Plus GX's load_rom does).
fn md_header(rom: &Path) -> Option<[u8; 0x200]> {
    let len = crate::archive::len(rom).ok()?;
    let mut h = [0u8; 0x200];
    crate::archive::read_at(rom, 0, &mut h).ok()?;
    let smd = &h[0x100..0x104] != b"SEGA" && len % 512 == 0 && (len / 512) % 2 == 1;
    if smd {
        let mut block = vec![0u8; 0x4000];
        crate::archive::read_at(rom, 0x200, &mut block).ok()?;
        for i in 0..0x100 {
            h[2 * i] = block[0x2000 + i];
            h[2 * i + 1] = block[i];
        }
    }
    h[0x100..0x110]
        .windows(4)
        .any(|w| w == b"SEGA")
        .then_some(h)
}

/// The regions a Mega Drive header names, as bits: 1 Japan, 2 Japan PAL, 4 USA, 8 Europe.
/// None when the file has no Mega Drive header or its country field names none.
pub fn md_region(rom: &Path) -> Option<u8> {
    let h = md_header(rom)?;
    let country = &h[0x1f0..0x200];
    let word = |w: &[u8]| country[..3].eq_ignore_ascii_case(w);
    let bits = if word(b"eur") {
        8
    } else if word(b"jap") {
        1
    } else if word(b"usa") {
        4
    } else {
        country[..4].iter().fold(0u8, |bits, c| {
            bits | match c.to_ascii_uppercase() {
                b'U' => 4,
                b'J' | b'K' => 1,
                b'E' => 8,
                c @ 0..=15 => c,
                c @ b'0'..=b'9' => c - b'0',
                c @ b'A'..=b'F' => c - b'A' + 10,
                _ => 0,
            }
        })
    };
    (bits != 0).then_some(bits)
}

/// The Japanese cart when the header names only Japan, the Western one when it names only the
/// West; None when it names both or nothing, leaving it to the game's name.
pub fn md_japan(rom: &Path) -> Option<bool> {
    let bits = md_region(rom)?;
    match (bits & 0b0011 != 0, bits & 0b1100 != 0) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        _ => None,
    }
}
