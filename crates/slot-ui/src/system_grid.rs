//! A transparent system picker, composed over the shelf's existing wallpaper.
use std::path::Path;

use slot_gfx::{OUT_H, OUT_W};
use slot_store::Platform;

use crate::{art, text, UndoFace};

pub const SYSTEM_ORDER: [Platform; 14] = [
    Platform::Gb,
    Platform::Gbc,
    Platform::Gba,
    Platform::Nes,
    Platform::Snes,
    Platform::Md,
    Platform::S32x,
    Platform::SegaCd,
    Platform::Ps1,
    Platform::Gg,
    Platform::Sms,
    Platform::N64,
    Platform::Ngp,
    Platform::Pico8,
];
const PAGE: usize = 6;
const INK: [u8; 3] = [246, 244, 239];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemGrid {
    pub entries: Vec<(Platform, usize)>,
    pub selected: usize,
}

impl SystemGrid {
    pub fn step(&mut self, by: i32) {
        if self.entries.is_empty() {
            return;
        }
        let last = self.entries.len() - 1;
        self.selected = match by {
            -3 if self.selected >= 3 => self.selected - 3,
            3 if self.selected / 3 < last / 3 => (self.selected + 3).min(last),
            -3 | 3 => self.selected,
            _ => (self.selected as i32 + by).rem_euclid(self.entries.len() as i32) as usize,
        };
    }

    pub fn page(&mut self, by: i32) {
        if self.entries.is_empty() {
            return;
        }
        let pages = self.entries.len().div_ceil(PAGE);
        let page = (self.selected as i32 / PAGE as i32 + by).rem_euclid(pages as i32) as usize;
        self.selected = (page * PAGE + self.selected % PAGE).min(self.entries.len() - 1);
    }

    pub fn face(&self, icons: &SystemIcons) -> UndoFace {
        let mut face = UndoFace {
            rgba: vec![0; (OUT_W * OUT_H * 4) as usize],
            w: OUT_W,
            h: OUT_H,
        };
        let page = self.selected / PAGE;
        for (i, (platform, count)) in self.entries.iter().enumerate().skip(page * PAGE).take(PAGE) {
            let cell = i % PAGE;
            let x = 18 + (cell % 3) as u32 * 204;
            let y = 76 + (cell / 3) as u32 * 166;
            let selected = i == self.selected;
            if selected {
                highlight(&mut face, x, y, 196, 158);
            }
            if let Some(icon) = &icons.0[platform.index()] {
                composite(
                    &mut face,
                    icon,
                    x + (196 - icon.w) / 2,
                    y + 10 + (96 - icon.h) / 2,
                );
            } else {
                // An incomplete card still has a usable, named system tile.
                line(
                    &mut face,
                    platform.dir_name(),
                    x + 4,
                    y + 36,
                    188,
                    40,
                    24.0,
                    INK,
                );
            }
            line(
                &mut face,
                name(*platform),
                x + 4,
                y + 110,
                188,
                22,
                15.0,
                INK,
            );
            if selected {
                let count = if *count == 1 {
                    "1 game".into()
                } else {
                    format!("{count} games")
                };
                line(
                    &mut face,
                    &count,
                    x + 4,
                    y + 133,
                    188,
                    18,
                    12.0,
                    [184, 181, 175],
                );
            }
        }
        if self.entries.is_empty() {
            line(&mut face, "No games found", 40, 180, 560, 40, 24.0, INK);
            line(
                &mut face,
                "Add games to Slot/Games",
                40,
                220,
                560,
                30,
                16.0,
                INK,
            );
        } else if self.entries.len() > PAGE {
            line(
                &mut face,
                &format!(
                    "L1   {} / {}   R1",
                    page + 1,
                    self.entries.len().div_ceil(PAGE)
                ),
                220,
                412,
                200,
                20,
                12.0,
                INK,
            );
        }
        // Leave the shelf's live battery, Wi-Fi and clock at their existing positions.
        for y in 440..OUT_H {
            for x in 0..OUT_W {
                face.rgba[((y * OUT_W + x) * 4 + 3) as usize] = 115;
            }
        }
        line(
            &mut face,
            "A Open    X Search",
            175,
            444,
            290,
            28,
            14.0,
            INK,
        );
        face
    }
}

fn name(platform: Platform) -> &'static str {
    match platform {
        Platform::Nes => "Nintendo / NES",
        Platform::Snes => "Super Nintendo",
        Platform::Md => "Mega Drive",
        Platform::Gg => "Game Gear",
        Platform::Sms => "Master System",
        _ => platform.name(),
    }
}

/// Read once per frontend. Keep pixels sharp and remove only transparent canvas padding.
#[derive(Default)]
pub struct SystemIcons([Option<UndoFace>; Platform::ALL.len()]);

impl SystemIcons {
    pub fn load(root: &Path) -> Self {
        Self(Platform::ALL.map(|p| {
            let (rgba, w, h) = art::decode(
                &root
                    .join("System/Icons")
                    .join(format!("{}.png", p.dir_name())),
            )?;
            let mut bounds = (w, h, 0, 0);
            for y in 0..h {
                for x in 0..w {
                    if rgba[((y * w + x) * 4 + 3) as usize] != 0 {
                        bounds = (
                            bounds.0.min(x),
                            bounds.1.min(y),
                            bounds.2.max(x + 1),
                            bounds.3.max(y + 1),
                        );
                    }
                }
            }
            let (left, top, right, bottom) = bounds;
            if right <= left || bottom <= top {
                return None;
            }
            let (sw, sh) = (right - left, bottom - top);
            let scale = 96.0 / sw.max(sh) as f32;
            // Integer enlargement preserves each source pixel's square shape.
            let scale = if scale >= 1.0 { scale.floor() } else { scale };
            let (dw, dh) = (
                (sw as f32 * scale).round() as u32,
                (sh as f32 * scale).round() as u32,
            );
            let mut out = vec![0; (dw * dh * 4) as usize];
            for y in 0..dh {
                for x in 0..dw {
                    let src = (((top + y * sh / dh) * w + left + x * sw / dw) * 4) as usize;
                    let dst = ((y * dw + x) * 4) as usize;
                    out[dst..dst + 4].copy_from_slice(&rgba[src..src + 4]);
                }
            }
            Some(UndoFace {
                rgba: out,
                w: dw,
                h: dh,
            })
        }))
    }
}

fn line(dst: &mut UndoFace, value: &str, x: u32, y: u32, w: u32, h: u32, px: f32, ink: [u8; 3]) {
    let Some(font) = text::label_font() else {
        return;
    };
    let layout = text::fit(font, value, w as f32, 1, px, 10.0);
    let mut face = UndoFace {
        rgba: vec![0; (w * h * 4) as usize],
        w,
        h,
    };
    text::draw_centred(&mut face.rgba, w, h, &layout, ink);
    composite(dst, &face, x, y);
}

fn composite(dst: &mut UndoFace, src: &UndoFace, x: u32, y: u32) {
    for sy in 0..src.h.min(dst.h.saturating_sub(y)) {
        for sx in 0..src.w.min(dst.w.saturating_sub(x)) {
            let s = ((sy * src.w + sx) * 4) as usize;
            let d = (((y + sy) * dst.w + x + sx) * 4) as usize;
            let a = src.rgba[s + 3] as u32;
            let under = dst.rgba[d + 3] as u32 * (255 - a) / 255;
            let alpha = a + under;
            if alpha == 0 {
                continue;
            }
            for c in 0..3 {
                dst.rgba[d + c] =
                    ((src.rgba[s + c] as u32 * a + dst.rgba[d + c] as u32 * under) / alpha) as u8;
            }
            dst.rgba[d + 3] = alpha as u8;
        }
    }
}

fn highlight(dst: &mut UndoFace, x: u32, y: u32, w: u32, h: u32) {
    for dy in 0..h {
        for dx in 0..w {
            let px = dx as f32 + 0.5;
            let py = dy as f32 + 0.5;
            let distance =
                (px - px.clamp(10.0, w as f32 - 10.0)).hypot(py - py.clamp(10.0, h as f32 - 10.0));
            if distance > 10.0 {
                continue;
            }
            let edge = distance > 8.5 || dx < 2 || dy < 2 || dx >= w - 2 || dy >= h - 2;
            let at = (((y + dy) * dst.w + x + dx) * 4) as usize;
            dst.rgba[at..at + 4].copy_from_slice(if edge {
                &[230, 227, 218, 220]
            } else {
                &[0, 0, 0, 55]
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shipped_system_icon_decodes_and_background_stays_transparent() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../brick/card/Slot");
        let icons = SystemIcons::load(&root);
        assert!(icons.0.iter().all(|i| i
            .as_ref()
            .is_some_and(|f| f.w > 0 && f.w <= 96 && f.h > 0 && f.h <= 96)));
        for extra in ["32X", "SEGACD", "PCE", "PCECD", "NEOGEO"] {
            assert!(art::decode(&root.join(format!("System/Icons/{extra}.png"))).is_some());
        }
        let mut grid = SystemGrid {
            entries: SYSTEM_ORDER.map(|p| (p, 12)).to_vec(),
            selected: 2,
        };
        for _ in 0..3 {
            let face = grid.face(&icons);
            assert_eq!(face.rgba.len(), (OUT_W * OUT_H * 4) as usize);
            assert!(
                face.rgba[..(OUT_W * 76 * 4) as usize]
                    .iter()
                    .all(|v| *v == 0),
                "no title or baked-in wallpaper"
            );
            grid.page(1);
        }
    }
}
