use crate::lcd3x::ScreenEffect;
use crate::power::{screen_brightness, screen_rect};
use crate::quad::Quad;
use crate::shaders::{GAME_FRAG, RECT_VERT};
use crate::surface::{GfxError, OUT_H, OUT_W};

pub const SCALE: u32 = 3;
// Emulator frames default to a 240x160 canvas (GBA), independent of UI layout; a NES
// session sets its own with set_canvas. Source and picture rects are in canvas units.
pub const SRC_W: u32 = 240;
pub const SRC_H: u32 = 160;

pub const WHOLE_TEXTURE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

pub struct GamePass {
    prog: gl::types::GLuint,
    game: gl::types::GLuint,
    u_rect: gl::types::GLint,
    u_bright: gl::types::GLint,
    u_uv: gl::types::GLint,
    u_mode: gl::types::GLint,
    u_paper_size: gl::types::GLint,
    u_fbo: gl::types::GLint,
    fbo_scale: [f32; 2],
    u_pic: gl::types::GLint,
    pic: [f32; 4],
    paper: gl::types::GLuint,
    paper_size: f32,
    effect: ScreenEffect,
    power: f32,
    src: [f32; 4],
    /// The active picture's native pixel size, if per-game integer scale is on.
    integer_picture: Option<(u32, u32)>,
    canvas: (u32, u32),
    u_src: gl::types::GLint,
}

impl GamePass {
    pub fn new() -> Result<Self, GfxError> {
        let prog = crate::shaders::program(RECT_VERT, GAME_FRAG)?;
        let game = crate::gl::texture(SRC_W, SRC_H, gl::NEAREST, gl::CLAMP_TO_EDGE, gl::BGRA, None);
        let paper = crate::gl::texture(1, 1, gl::NEAREST, gl::REPEAT, gl::RGBA, Some(&[255; 4]));
        let (u_rect, u_bright, u_uv, u_mode, u_paper_size, u_pic, u_fbo, u_src);
        unsafe {
            gl::UseProgram(prog);
            gl::Uniform1i(crate::gl::uniform_location(prog, "u_game"), 0);
            u_src = crate::gl::uniform_location(prog, "u_src");
            gl::Uniform2f(u_src, SRC_W as f32, SRC_H as f32);
            gl::Uniform2f(
                crate::gl::uniform_location(prog, "u_target"),
                OUT_W as f32,
                OUT_H as f32,
            );
            u_rect = crate::gl::uniform_location(prog, "u_rect");
            u_bright = crate::gl::uniform_location(prog, "u_bright");
            u_uv = crate::gl::uniform_location(prog, "u_uv");
            gl::Uniform1i(crate::gl::uniform_location(prog, "u_paper"), 1);
            u_mode = crate::gl::uniform_location(prog, "u_mode");
            u_paper_size = crate::gl::uniform_location(prog, "u_paper_size");
            u_fbo = crate::gl::uniform_location(prog, "u_fbo");
            u_pic = crate::gl::uniform_location(prog, "u_pic");
        }
        Ok(GamePass {
            prog,
            game,
            u_rect,
            u_bright,
            u_uv,
            u_mode,
            u_paper_size,
            u_fbo,
            fbo_scale: [1.0, 1.0],
            u_pic,
            pic: WHOLE_TEXTURE,
            paper,
            paper_size: 1.0,
            effect: ScreenEffect::Lcd3x,
            power: 1.0,
            src: WHOLE_TEXTURE,
            integer_picture: None,
            canvas: (SRC_W, SRC_H),
            u_src,
        })
    }

    pub fn set_power(&mut self, t: f32) {
        self.power = t.clamp(0.0, 1.0);
    }

    pub fn set_picture(&mut self, rect: [f32; 4]) {
        self.pic = rect;
    }

    pub fn set_fbo_scale(&mut self, x: f32, y: f32) {
        self.fbo_scale = [x, y];
    }

    pub fn set_effect(&mut self, effect: ScreenEffect) {
        self.effect = effect;
    }

    pub fn set_paper(&mut self, size: u32, rgba: &[u8]) {
        let tex = crate::gl::texture(size, size, gl::NEAREST, gl::REPEAT, gl::RGBA, Some(rgba));
        unsafe { gl::DeleteTextures(1, &self.paper) };
        self.paper = tex;
        self.paper_size = size as f32;
    }

    pub fn set_source_rect(&mut self, rect: [f32; 4]) {
        self.src = rect;
    }

    pub fn set_integer_picture(&mut self, pixel_size: Option<(u32, u32)>) {
        self.integer_picture = pixel_size;
    }

    /// The canvas frames arrive in; the game texture is remade to fit when it changes.
    pub fn set_canvas(&mut self, w: u32, h: u32) {
        if (w, h) == self.canvas || w == 0 || h == 0 {
            return;
        }
        unsafe {
            gl::DeleteTextures(1, &self.game);
            gl::UseProgram(self.prog);
            gl::Uniform2f(self.u_src, w as f32, h as f32);
        }
        self.game = crate::gl::texture(w, h, gl::NEAREST, gl::CLAMP_TO_EDGE, gl::BGRA, None);
        self.canvas = (w, h);
    }

    pub fn upload(&mut self, xrgb8888: &[u8]) {
        // A frame from another canvas (the core before a switch between a NES and a Game Boy)
        // is skipped rather than drawn out of shape.
        let (w, h) = self.canvas;
        if xrgb8888.len() != (w * h * 4) as usize {
            return;
        }
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.game);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                w as i32,
                h as i32,
                gl::BGRA,
                gl::UNSIGNED_BYTE,
                xrgb8888.as_ptr() as *const std::ffi::c_void,
            );
        }
    }

    pub fn draw(&self, quad: &Quad) {
        let src = match self.effect {
            ScreenEffect::Grid if !cfg!(feature = "brick") => {
                whole_scale_source(self.src, self.fbo_scale, self.canvas)
            }
            _ => self.src,
        };
        self.draw_source(self.game, quad, src);
    }

    pub fn draw_still(&self, tex: gl::types::GLuint, quad: &Quad) {
        self.draw_source(
            tex,
            quad,
            if cfg!(feature = "brick") {
                self.src
            } else {
                WHOLE_TEXTURE
            },
        );
    }

    fn draw_source(&self, tex: gl::types::GLuint, quad: &Quad, src: [f32; 4]) {
        let [x, y, w, h] = self.integer_picture
            .map(|pixels| integer_game_rect(pixels, self.fbo_scale, self.power))
            .unwrap_or_else(|| {
                let (x,y,w,h) = screen_rect(self.power);
                [x,y,w,h]
            });
        unsafe {
            gl::UseProgram(self.prog);
            gl::Uniform4f(self.u_rect, x, y, w, h);
            gl::Uniform4f(self.u_uv, src[0], src[1], src[2], src[3]);
            gl::Uniform1f(self.u_bright, screen_brightness(self.power));
            gl::Uniform1f(self.u_mode, self.effect.mode());
            gl::Uniform1f(self.u_paper_size, self.paper_size);
            gl::Uniform2f(self.u_fbo, self.fbo_scale[0], self.fbo_scale[1]);
            gl::Uniform4f(
                self.u_pic,
                self.pic[0],
                self.pic[1],
                self.pic[2],
                self.pic[3],
            );
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.paper);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, tex);
        }
        quad.draw();
    }
}

/// The largest **physical framebuffer** integer scale that fits the picture.
/// Compute in final panel pixels first (Brick Pro has a 1024x768 output, not
/// 640x480), then convert to the composited UI coordinate system. At full
/// power the picture starts on an integer physical pixel and occupies an exact
/// whole-number block of physical pixels for every source pixel.
/// When no 1x size fits (a tiny desktop window), fall back to normal fitting
/// rather than clipping the game.
pub fn integer_game_rect(
    pixels: (u32,u32),
    framebuffer_scale: [f32;2],
    power: f32,
) -> [f32;4] {
    let (pw,ph) = pixels;
    let [sx,sy] = framebuffer_scale;
    if pw==0 || ph==0 || !sx.is_finite() || !sy.is_finite() || sx<=0.0 || sy<=0.0 {
        let (x,y,w,h) = screen_rect(power);
        return [x,y,w,h];
    }
    let frame_w = (OUT_W as f32 * sx).round();
    let frame_h = (OUT_H as f32 * sy).round();
    let whole = (frame_w / pw as f32).floor()
        .min((frame_h / ph as f32).floor());
    if whole < 1.0 {
        let (x,y,w,h) = screen_rect(power);
        return [x,y,w,h];
    }
    let width = pw as f32 * whole;
    let height = ph as f32 * whole;
    let x = ((frame_w-width)*0.5).floor() / sx;
    let y = ((frame_h-height)*0.5).floor() / sy;
    let width = width / sx;
    let height = height / sy;
    // Preserve Slot's normal CRT-power animation on the game area.
    let (_,_,full_w,full_h) = screen_rect(power);
    let fx = full_w / OUT_W as f32;
    let fy = full_h / OUT_H as f32;
    [
        OUT_W as f32*0.5 + (x-OUT_W as f32*0.5)*fx,
        OUT_H as f32*0.5 + (y-OUT_H as f32*0.5)*fy,
        width*fx,
        height*fy,
    ]
}

fn whole_scale_source(src: [f32; 4], fbo: [f32; 2], canvas: (u32, u32)) -> [f32; 4] {
    let (x, w) = whole_scale(src[0], src[2], canvas.0 as f32, OUT_W as f32 * fbo[0]);
    let (y, h) = whole_scale(src[1], src[3], canvas.1 as f32, OUT_H as f32 * fbo[1]);
    [x, y, w, h]
}

fn whole_scale(at: f32, len: f32, texels: f32, out: f32) -> (f32, f32) {
    let shown = len * texels;
    let scale = (out / shown).ceil();
    let trim = scale * shown - out;
    if trim >= 0.1 * shown {
        return (at, len);
    }
    let top = (trim / 2.0).floor();
    (at + top / scale / texels, out / scale / texels)
}

impl Drop for GamePass {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.game);
            gl::DeleteTextures(1, &self.paper);
            gl::DeleteProgram(self.prog);
        }
    }
}

#[cfg(test)]
mod integer_tests {
    use super::*;
    fn assert_rect(actual:[f32;4],expected:[f32;4]){
        for (a,b) in actual.into_iter().zip(expected) {
            assert!((a-b).abs()<0.0005,"expected {b}, got {a}");
        }
    }

    #[test]
    fn brick_640_integer_scale_centres_gb_gba_nes_and_md_without_crop() {
        // OUT_W and OUT_H are 640x480 for a Brick build.
        if OUT_W!=640 {return;}
        assert_rect(integer_game_rect((160,144),[1.0,1.0],1.0),[80.0,24.0,480.0,432.0]);
        assert_rect(integer_game_rect((240,160),[1.0,1.0],1.0),[80.0,80.0,480.0,320.0]);
        assert_rect(integer_game_rect((256,224),[1.0,1.0],1.0),[64.0,16.0,512.0,448.0]);
        assert_rect(integer_game_rect((320,224),[1.0,1.0],1.0),[0.0,16.0,640.0,448.0]);
    }

    #[test]
    fn hires_brick_pro_uses_integer_physical_pixels_not_fractional_ui_pixels() {
        if OUT_W!=640 {return;}
        let rect=integer_game_rect((240,160),[1.6,1.6],1.0);
        let scaled=[rect[0]*1.6,rect[1]*1.6,rect[2]*1.6,rect[3]*1.6];
        assert_rect(scaled,[32.0,64.0,960.0,640.0]);
    }

    #[test]
    fn tiny_panel_fits_instead_of_cropping_when_integer_is_impossible() {
        let [x,y,w,h] = integer_game_rect((640,480),[0.5,0.5],1.0);
        assert_rect([x,y,w,h],[0.0,0.0,OUT_W as f32,OUT_H as f32]);
    }
}
