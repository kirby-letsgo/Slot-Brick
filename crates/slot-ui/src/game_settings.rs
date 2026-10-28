//! Compact per-game emulator settings panel, also used over the pause menu.
//! Drawing is pure; no config, ROM or filesystem work happens here.
use crate::{text, UndoFace, OUT_W, OUT_H};

const INK: [u8; 3] = [246, 244, 239];
const MUTED: [u8; 3] = [171, 171, 178];
const ROWS: usize = 7;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameSettingsView {
    pub game: String,
    pub rows: Vec<(String, String)>,
    pub selected: usize,
    pub from_pause: bool,
}

impl GameSettingsView {
    pub fn face(&self) -> UndoFace {
        let mut face=UndoFace { rgba:vec![0;(OUT_W*OUT_H*4) as usize], w:OUT_W, h:OUT_H };
        for px in face.rgba.chunks_exact_mut(4) {
            px.copy_from_slice(&[13, 13, 17, 255]);
        }
        put(&mut face, "EMULATOR SETTINGS", 28, 20, 580, 32, 28.0, INK);
        put(&mut face, &self.game, 28, 57, 580, 25, 18.0, MUTED);
        let start=self.selected / ROWS * ROWS;
        for (i,(name,value)) in self.rows.iter().enumerate().skip(start).take(ROWS) {
            let y=94+(i-start) as u32*43;
            if self.selected==i {
                for yy in y..y+40 {
                    for xx in 20..620 {
                        let at=((yy*OUT_W+xx)*4) as usize;
                        face.rgba[at..at+4].copy_from_slice(&[66, 65, 73, 255]);
                    }
                }
            }
            put(&mut face, name, 32,y+4,282,30,20.0,INK);
            put(&mut face, value, 323,y+4,286,30,18.0,
                if self.selected==i { INK } else { MUTED });
        }
        put(&mut face, &format!("{} / {}",
            self.selected / ROWS+1, self.rows.len().div_ceil(ROWS).max(1)),
            32,400,575,22,14.0,MUTED);
        put(&mut face, "UP/DOWN  SELECT      LEFT/RIGHT  CHANGE      B  BACK",
            20,441,600,24,14.0,INK);
        face
    }
}

#[allow(clippy::too_many_arguments)]
fn put(dst:&mut UndoFace,text_value:&str,x:u32,y:u32,w:u32,h:u32,px:f32,ink:[u8;3]) {
    let Some(font)=text::label_font() else { return };
    let layout=text::fit(font,text_value,w as f32,1,px,12.0);
    let mut rgba=vec![0;(w*h*4) as usize];
    text::draw_centred(&mut rgba,w,h,&layout,ink);
    for yy in 0..h {
        for xx in 0..w {
            let src=((yy*w+xx)*4) as usize;
            let dst_at=(((y+yy)*OUT_W+x+xx)*4) as usize;
            let a=rgba[src+3] as u32;
            if a==0 { continue }
            for c in 0..3 {
                dst.rgba[dst_at+c]=((rgba[src+c] as u32*a+
                    dst.rgba[dst_at+c] as u32*(255-a))/255) as u8;
            }
            dst.rgba[dst_at+3]=255;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_pages_render_at_the_active_output_canvas_size() {
        let mut view=GameSettingsView {
            game:"Sonic (Game Gear)".into(),
            rows:(0..10).map(|i|(format!("Option {i}"),"SYSTEM DEFAULT".into())).collect(),
            selected:0,from_pause:true
        };
        let first=view.face();
        assert_eq!((first.w,first.h),(OUT_W,OUT_H));
        assert_eq!(first.rgba.len(),(OUT_W*OUT_H*4) as usize);
        view.selected=8;
        let second=view.face();
        assert_ne!(first.rgba,second.rgba);
    }
}
