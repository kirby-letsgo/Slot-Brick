use slot_gfx::{Draw, TexId, OUT_H, OUT_W};
use slot_power::Battery;

use crate::battery::{draw_gauge, GAUGE_H};
use crate::plate::HINT_H;
use crate::slot_chrome::MOUTH_H;

const FOOTER_Y: f32 = OUT_H as f32 - MOUTH_H + (MOUTH_H - HINT_H as f32) / 2.0;
const FOOTER_MARGIN: f32 = 24.0;

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct Printed {
    pub face: Option<TexId>,
    pub w: u32,
}

impl Printed {
    pub fn new(face: TexId, w: u32) -> Self {
        Printed {
            face: Some(face),
            w,
        }
    }
}

/// How far the Wi-Fi symbol sits from the clock, and how big it is drawn.
const WIFI_GAP: f32 = 10.0;
pub const WIFI_PX: f32 = 18.0;

pub fn draw_footer(
    battery: Option<Battery>,
    percent: Printed,
    bolt: Option<TexId>,
    wifi: Option<(TexId, u32, u32)>,
    clock: Printed,
    out: &mut Vec<Draw>,
) {
    let y = FOOTER_Y + (HINT_H as f32 - GAUGE_H) / 2.0;
    draw_gauge(FOOTER_MARGIN, y, battery, percent, bolt, out);
    let clock_x = OUT_W as f32 - FOOTER_MARGIN - clock.w as f32;
    printed(clock_x, clock, out);
    // To the left of the clock: the two together say what the Brick is doing right now.
    if let Some((tex, w, h)) = wifi {
        out.push(Draw::Tex {
            x: (clock_x - WIFI_GAP - w as f32).round(),
            y: (FOOTER_Y + (HINT_H as f32 - h as f32) / 2.0).round(),
            w: w as f32,
            h: h as f32,
            tex,
            alpha: 1.0,
        });
    }
}

/// Keep the background FTP indicator visually separate from the clock and Wi-Fi.
/// It is an extra short piece of footer text, not another blocking menu or toast.
const TRANSFER_GAP: f32 = 12.0;

pub fn draw_footer_transfer(
    transfer: Printed,
    wifi: Option<(TexId, u32, u32)>,
    clock: Printed,
    out: &mut Vec<Draw>,
) {
    if transfer.face.is_none() || transfer.w == 0 { return; }
    let clock_x = OUT_W as f32 - FOOTER_MARGIN - clock.w as f32;
    let left_edge = wifi.map_or(clock_x, |(_, w, _)| clock_x - WIFI_GAP - w as f32);
    let x = (left_edge - TRANSFER_GAP - transfer.w as f32).round();
    printed(x, transfer, out);
}

/// How far a note sits from the battery's percent.
const NOTE_GAP: f32 = 22.0;

/// A note beside the battery's percent (the selected game's play time, on the shelf).
pub fn draw_footer_note(percent: Printed, note: Printed, out: &mut Vec<Draw>) {
    let x = crate::battery::gauge_end(FOOTER_MARGIN, percent) + NOTE_GAP;
    printed(x.round(), note, out);
}

pub(crate) fn draw_printed(x: f32, y: f32, p: Printed, out: &mut Vec<Draw>) {
    if p.w == 0 {
        return;
    }
    let (w, h) = (p.w as f32, HINT_H as f32);
    out.push(match p.face {
        Some(tex) => Draw::Tex {
            x,
            y,
            w,
            h,
            tex,
            alpha: 1.0,
        },
        None => Draw::Rect {
            x,
            y,
            w,
            h,
            colour: [1.0, 1.0, 1.0, 0.08],
        },
    });
}

fn printed(x: f32, p: Printed, out: &mut Vec<Draw>) {
    draw_printed(x, FOOTER_Y, p, out);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_ftp_label_sits_left_of_wifi_and_clock() {
        let clock = Printed::new(TexId::from_raw(1), 52);
        let wifi = Some((TexId::from_raw(2), 18, 18));
        let transfer = Printed::new(TexId::from_raw(3), 55);
        let mut draws = Vec::new();
        draw_footer_transfer(transfer, wifi, clock, &mut draws);
        assert_eq!(draws.len(), 1);
        match draws[0] {
            Draw::Tex { x, w, tex, .. } => {
                assert_eq!(tex, TexId::from_raw(3));
                assert_eq!(w, 55.0);
                let clock_x = OUT_W as f32 - FOOTER_MARGIN - 52.0;
                assert!(x + w <= clock_x - WIFI_GAP - 18.0 - TRANSFER_GAP);
            }
            _ => panic!("expected a text texture"),
        }
        draws.clear();
        draw_footer_transfer(Printed::default(), wifi, clock, &mut draws);
        assert!(draws.is_empty());
    }
}
