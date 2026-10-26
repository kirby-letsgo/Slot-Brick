//! Printed PC boxes for native ports. The shared SVG layers are rasterized once;
//! individual covers use the existing art worker/cache and never bake in wallpaper.
use std::sync::OnceLock;
use slot_store::Cart;
use crate::{art, text, CartFace};

pub const SIZE: (u32, u32) = (288, 405);
const PANEL: (u32, u32, u32, u32) = (36, 14, 238, 372);

fn layer(svg: &str) -> Vec<u8> {
    art::render_svg(svg, SIZE.0, SIZE.1).expect("embedded port box SVG")
}

fn over(face: &mut CartFace, rgba: &[u8], x: u32, y: u32, w: u32, h: u32) {
    for py in 0..h {
        for px in 0..w {
            let src = &rgba[((py*w+px)*4) as usize..][..4];
            let dst = &mut face.rgba[(((py+y)*face.w+px+x)*4) as usize..][..4];
            let a = src[3] as u32;
            let alpha = a*255 + dst[3] as u32*(255-a);
            if alpha == 0 { continue; }
            for c in 0..3 {
                dst[c] = ((src[c] as u32*a*255 + dst[c] as u32*dst[3] as u32*(255-a))/alpha) as u8;
            }
            dst[3] = ((alpha+127)/255) as u8;
        }
    }
}

pub fn face(cart: &Cart) -> CartFace {
    static BASE: OnceLock<Vec<u8>> = OnceLock::new();
    static FINISH: OnceLock<Vec<u8>> = OnceLock::new();
    let mut face = CartFace { w: SIZE.0, h: SIZE.1,
        rgba: BASE.get_or_init(|| layer(include_str!("../assets/ports-box.svg"))).clone() };
    let (x, y, w, h) = PANEL;
    let manager = cart.code == "SLOT_PORTMASTER";
    let cover = if manager { None } else { cart.label.as_ref().and_then(|p| art::contain(p, w, h)) };
    if let Some(cover) = cover {
        over(&mut face, &cover, x, y, w, h);
    } else {
        // The manager has a toolbox; missing or unreadable covers get a small game icon.
        let icon = if manager {
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="100"><path d="M40 28V16Q40 10 46 10H74Q80 10 80 16V28" fill="none" stroke="#e3cda6" stroke-width="8"/><rect x="12" y="28" width="96" height="62" rx="6" fill="#bd8c50"/><path d="M12 50H108" stroke="#694939" stroke-width="4"/><rect x="52" y="44" width="16" height="19" rx="2" fill="#f2dfb4"/></svg>"##
        } else {
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="100"><rect x="17" y="18" width="86" height="60" rx="5" fill="none" stroke="#e3cda6" stroke-width="5"/><path d="M43 48H63M53 38V58" stroke="#e3cda6" stroke-width="5"/><circle cx="82" cy="43" r="4" fill="#e3cda6"/><circle cx="90" cy="55" r="4" fill="#e3cda6"/></svg>"##
        };
        if let Some(icon) = art::render_svg(icon, 108, 90) { over(&mut face, &icon, x+(w-108)/2, 90, 108, 90); }
        if let Some(font) = text::label_font() {
            let layout = text::fit(font, cart.display_name(), 209.0, 4, 29.0, 13.0);
            let mut title = vec![0; (w*135*4) as usize];
            text::draw_centred(&mut title, w, 135, &layout, [240, 222, 177]);
            over(&mut face, &title, x, 196, w, 135);
        }
    }
    if let Some(font) = text::label_font() {
        let layout = text::fit(font, cart.display_name(), 270.0, 1, 11.0, 7.0);
        let mut label = vec![0; 270*14*4];
        text::draw_centred(&mut label, 270, 14, &layout, [230, 207, 167]);
        let mut rotated = vec![0; label.len()];
        for y in 0..14usize { for x in 0..270usize {
            let src = (y*270+x)*4;
            let dst = ((269-x)*14+y)*4;
            rotated[dst..dst+4].copy_from_slice(&label[src..src+4]);
        }}
        over(&mut face, &rotated, 14, 84, 14, 270);
    }
    over(&mut face, FINISH.get_or_init(|| layer(include_str!("../assets/ports-box-finish.svg"))), 0, 0, SIZE.0, SIZE.1);
    face
}
