//! A small, paged setup report. The application supplies file checks; drawing never does I/O.
use crate::{text, UndoFace, OUT_H, OUT_W};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupStatus {
    Ready,
    Missing,
    Check,
    Optional,
}
impl SetupStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "Files ready",
            Self::Missing => "Missing files",
            Self::Check => "Check files",
            Self::Optional => "Optional BIOS",
        }
    }
    fn ink(self) -> [u8; 3] {
        match self {
            Self::Ready => [153, 211, 170],
            Self::Missing | Self::Check => [244, 193, 122],
            Self::Optional => [188, 193, 205],
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetupEntry {
    pub name: String,
    pub status: SetupStatus,
    pub details: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemSetup {
    pub entries: Vec<SetupEntry>,
    pub selected: usize,
    pub detail: bool,
    pub detail_page: usize,
}
const ROWS: usize = 7;
const LINES: usize = 9;
const INK: [u8; 3] = [246, 244, 239];
impl SystemSetup {
    pub fn new(entries: Vec<SetupEntry>) -> Self {
        Self {
            entries,
            selected: 0,
            detail: false,
            detail_page: 0,
        }
    }
    pub fn step(&mut self, by: i32) {
        if self.detail {
            let pages = self.detail_lines().len().div_ceil(LINES).max(1);
            self.detail_page =
                (self.detail_page as i32 + by.signum()).clamp(0, pages as i32 - 1) as usize;
        } else if !self.entries.is_empty() {
            self.selected =
                (self.selected as i32 + by).rem_euclid(self.entries.len() as i32) as usize;
        }
    }
    pub fn open(&mut self) {
        self.detail = !self.entries.is_empty();
        self.detail_page = 0;
    }
    pub fn back(&mut self) -> bool {
        if self.detail {
            self.detail = false;
            true
        } else {
            false
        }
    }
    fn detail_lines(&self) -> Vec<String> {
        self.entries
            .get(self.selected)
            .map(|e| {
                e.details
                    .iter()
                    .flat_map(|s| wrap(s, 18.0, 568.0))
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn face(&self) -> UndoFace {
        let mut f = UndoFace {
            rgba: vec![0; (OUT_W * OUT_H * 4) as usize],
            w: OUT_W,
            h: OUT_H,
        };
        // Match the Settings background and reserve a fixed footer for controls.
        for p in f.rgba.chunks_exact_mut(4) {
            p.copy_from_slice(&[8, 8, 10, 255]);
        }
        if self.detail {
            if let Some(e) = self.entries.get(self.selected) {
                line(&mut f, &e.name.to_uppercase(), 36, 28, 568, 26, 24.0, INK);
                line(
                    &mut f,
                    e.status.label(),
                    36,
                    62,
                    568,
                    24,
                    16.0,
                    e.status.ink(),
                );
                let lines = self.detail_lines();
                let page = self
                    .detail_page
                    .min(lines.len().div_ceil(LINES).saturating_sub(1));
                for (i, s) in lines.iter().skip(page * LINES).take(LINES).enumerate() {
                    left_line(&mut f, s, 36, 108 + i as u32 * 30, 568, 26, 18.0, INK);
                }
                line(
                    &mut f,
                    &format!("{} / {}", page + 1, lines.len().div_ceil(LINES).max(1)),
                    36,
                    399,
                    568,
                    22,
                    13.0,
                    [160; 3],
                );
            }
            line(
                &mut f,
                "B  BACK     UP / DOWN  PAGE     X  RECHECK",
                24,
                441,
                592,
                22,
                13.0,
                INK,
            );
        } else {
            line(&mut f, "SYSTEM SETUP", 36, 25, 568, 28, 24.0, INK);
            let start = self.selected / ROWS * ROWS;
            for (i, e) in self.entries.iter().enumerate().skip(start).take(ROWS) {
                let y = 84 + (i - start) as u32 * 44;
                if i == self.selected {
                    for yy in y..y + 40 {
                        for xx in 24..616 {
                            let at = ((yy * OUT_W + xx) * 4) as usize;
                            f.rgba[at..at + 4].copy_from_slice(&[62, 62, 70, 255]);
                        }
                    }
                }
                left_line(&mut f, &e.name, 36, y + 6, 350, 28, 20.0, INK);
                line(
                    &mut f,
                    e.status.label(),
                    398,
                    y + 6,
                    206,
                    28,
                    16.0,
                    e.status.ink(),
                );
            }
            line(
                &mut f,
                &format!(
                    "{} / {}",
                    self.selected / ROWS + 1,
                    self.entries.len().div_ceil(ROWS).max(1)
                ),
                36,
                403,
                568,
                22,
                13.0,
                [160; 3],
            );
            line(
                &mut f,
                "B  BACK     A  DETAILS     X  RECHECK",
                24,
                441,
                592,
                22,
                14.0,
                INK,
            );
        }
        f
    }
}
// Keep the case of filenames and paths, including when a long word must wrap.
fn wrap(s: &str, px: f32, width: f32) -> Vec<String> {
    let Some(font) = text::label_font() else {
        return vec![s.into()];
    };
    let mut out = Vec::new();
    let mut line = String::new();
    for word in s.split_whitespace() {
        let next = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if text::line_width(font, &next, px, 0.0) <= width {
            line = next;
            continue;
        }
        if !line.is_empty() {
            out.push(std::mem::take(&mut line));
        }
        for ch in word.chars() {
            let next = format!("{line}{ch}");
            if !line.is_empty() && text::line_width(font, &next, px, 0.0) > width {
                out.push(std::mem::take(&mut line));
            }
            line.push(ch);
        }
    }
    if !line.is_empty() || out.is_empty() {
        out.push(line);
    }
    out
}
#[allow(clippy::too_many_arguments)]
fn left_line(
    dst: &mut UndoFace,
    s: &str,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    mut px: f32,
    ink: [u8; 3],
) {
    let Some(font) = text::label_font() else {
        return;
    };
    while px > 10.0 && text::line_width(font, s, px, 0.0) > w as f32 {
        px -= 1.0;
    }
    let tight = (text::line_width(font, s, px, 0.0).ceil() as u32 + 2).min(w);
    line(dst, s, x, y, tight, h, px, ink);
}
#[allow(clippy::too_many_arguments)]
fn line(dst: &mut UndoFace, s: &str, x: u32, y: u32, w: u32, h: u32, mut px: f32, ink: [u8; 3]) {
    let Some(font) = text::label_font() else {
        return;
    };
    while px > 10.0 && text::line_width(font, s, px, 0.0) > w as f32 {
        px -= 1.0;
    }
    let layout = text::Layout {
        lines: vec![s.into()],
        px,
        tracking: 0.0,
    };
    let mut rgba = vec![0; (w * h * 4) as usize];
    text::draw_centred(&mut rgba, w, h, &layout, ink);
    for sy in 0..h.min(dst.h - y) {
        for sx in 0..w.min(dst.w - x) {
            let src = ((sy * w + sx) * 4) as usize;
            let at = (((sy + y) * dst.w + sx + x) * 4) as usize;
            let a = rgba[src + 3] as u32;
            for c in 0..3 {
                dst.rgba[at + c] =
                    ((rgba[src + c] as u32 * a + dst.rgba[at + c] as u32 * (255 - a)) / 255) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_rows_are_reachable_and_long_details_keep_their_exact_case() {
        let mut view = SystemSetup::new(
            (0..17)
                .map(|i| SetupEntry {
                    name: format!("System {i}"),
                    status: SetupStatus::Ready,
                    details: vec!["Slot/BIOS/PICO8/pico8_64".into(); 20],
                })
                .collect(),
        );
        view.step(-1);
        assert_eq!(view.selected, 16);
        view.open();
        view.step(1);
        assert_eq!(view.detail_page, 1);
        assert_eq!(view.detail_lines()[0], "Slot/BIOS/PICO8/pico8_64");
        for _ in 0..20 {
            view.step(1);
        }
        assert_eq!(view.detail_page, 2);
        let face = view.face();
        assert_eq!(face.rgba.len(), (OUT_W * OUT_H * 4) as usize);
        assert!(view.back());
        assert_eq!(view.selected, 16);
        view.step(1);
        assert_eq!(view.selected, 0);
    }
    #[test]
    fn long_unbroken_filenames_are_wrapped_without_losing_characters() {
        let path = "Slot/BIOS/".to_string() + &"Abc_123".repeat(40) + ".bin";
        let lines = wrap(&path, 18.0, 568.0);
        assert!(lines.len() > 1);
        assert_eq!(lines.concat(), path);
    }
}
