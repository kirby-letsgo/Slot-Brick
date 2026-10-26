//! Render the actual cartridge geometry. Optional second argument supplies n64-label.png
//! and ngp-label.png (for example from cart_art::tests::real_new_console_art_and_previews).
use slot_store::{Cart, Platform};
fn main() {
    let mut args = std::env::args_os().skip(1);
    let out = std::path::PathBuf::from(args.next().expect("output directory"));
    let labels = args.next().map(std::path::PathBuf::from);
    std::fs::create_dir_all(&out).unwrap();
    for (platform, stem) in [
        (Platform::N64, "Nintendo 64"),
        (Platform::Ngp, "Neo Geo Pocket Color"),
        (Platform::Ws, "WonderSwan"),
        (Platform::Lynx, "Atari Lynx"),
    ] {
        let label = labels
            .as_ref()
            .map(|dir| dir.join(format!("{}-label.png", platform.dir_name().to_lowercase())));
        let cart = Cart {
            platform,
            stem: stem.into(),
            rom: "preview".into(),
            label,
            famicom_label: None,
            title: String::new(),
            code: String::new(),
            shell: None,
        };
        let face = slot_ui::cart_face(&cart);
        // An opaque, padded preview preserves the silhouette in clients that ignore PNG alpha.
        let (w, h) = (face.w + 48, face.h + 48);
        let mut rgb = vec![0; (w * h * 3) as usize];
        for pixel in rgb.chunks_exact_mut(3) {
            pixel.copy_from_slice(&[242, 240, 235]);
        }
        for y in 0..face.h {
            for x in 0..face.w {
                let src = ((y * face.w + x) * 4) as usize;
                let dst = (((y + 24) * w + x + 24) * 3) as usize;
                let a = face.rgba[src + 3] as u32;
                for c in 0..3 {
                    rgb[dst + c] =
                        ((face.rgba[src + c] as u32 * a + rgb[dst + c] as u32 * (255 - a) + 127)
                            / 255) as u8;
                }
            }
        }
        let mut e = png::Encoder::new(
            std::fs::File::create(out.join(format!("{}.png", platform.dir_name()))).unwrap(),
            w,
            h,
        );
        e.set_color(png::ColorType::Rgb);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(&rgb).unwrap();
    }
}
