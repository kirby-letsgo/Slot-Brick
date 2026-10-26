//! A dedicated GLES2 pbuffer on the emulation thread. The shelf's EGL context stays independent.
//! Loading EGL dynamically keeps software cores and host builds independent of GPU libraries.
use libloading::Library;
use std::ffi::{c_char, c_void};
type Ptr = *mut c_void;
type Display = unsafe extern "C" fn(Ptr) -> Ptr;
type Initialize = unsafe extern "C" fn(Ptr, *mut i32, *mut i32) -> u32;
type BindApi = unsafe extern "C" fn(u32) -> u32;
type Choose = unsafe extern "C" fn(Ptr, *const i32, *mut Ptr, i32, *mut i32) -> u32;
type Surface = unsafe extern "C" fn(Ptr, Ptr, *const i32) -> Ptr;
type Context = unsafe extern "C" fn(Ptr, Ptr, Ptr, *const i32) -> Ptr;
type Current = unsafe extern "C" fn(Ptr, Ptr, Ptr, Ptr) -> u32;
type Destroy = unsafe extern "C" fn(Ptr, Ptr) -> u32;
type Proc = unsafe extern "C" fn(*const c_char) -> *const c_void;
type ReadPixels = unsafe extern "C" fn(i32, i32, i32, i32, u32, u32, Ptr);
type BindFramebuffer = unsafe extern "C" fn(u32, u32);
type GetInteger = unsafe extern "C" fn(u32, *mut i32);
const NONE: i32 = 0x3038;
const MAX_SIDE: u32 = 640;
pub(crate) struct Hardware {
    _egl: Library,
    _gles: Library,
    display: Ptr,
    surface: Ptr,
    context: Ptr,
    current: Current,
    destroy_context: Destroy,
    destroy_surface: Destroy,
    proc: Proc,
    read: ReadPixels,
    bind: BindFramebuffer,
    get: GetInteger,
    framebuffer: u32,
    texture: u32,
    depthbuffer: u32,
    delete_framebuffers: unsafe extern "C" fn(i32, *const u32),
    delete_textures: unsafe extern "C" fn(i32, *const u32),
    delete_renderbuffers: unsafe extern "C" fn(i32, *const u32),
}
// A core is transferred to the emulation thread before load creates this context.
unsafe impl Send for Hardware {}
unsafe fn symbol<T: Copy>(lib: &Library, name: &[u8]) -> Result<T, String> {
    lib.get::<T>(name)
        .map(|s| *s)
        .map_err(|_| "Required GLES/EGL function is missing".into())
}
impl Hardware {
    pub fn open(depth: bool, stencil: bool) -> Result<Self, String> {
        unsafe {
            let egl = Library::new("libEGL.so.1")
                .or_else(|_| Library::new("libEGL.so"))
                .map_err(|_| "N64 needs the device's EGL driver")?;
            let gles = Library::new("libGLESv2.so.2")
                .or_else(|_| Library::new("libGLESv2.so"))
                .map_err(|_| "N64 needs the device's GLES2 driver")?;
            let display: Display = symbol(&egl, b"eglGetDisplay\0")?;
            let init: Initialize = symbol(&egl, b"eglInitialize\0")?;
            let bind_api: BindApi = symbol(&egl, b"eglBindAPI\0")?;
            let choose: Choose = symbol(&egl, b"eglChooseConfig\0")?;
            let create_surface: Surface = symbol(&egl, b"eglCreatePbufferSurface\0")?;
            let create_context: Context = symbol(&egl, b"eglCreateContext\0")?;
            let current: Current = symbol(&egl, b"eglMakeCurrent\0")?;
            let destroy_surface: Destroy = symbol(&egl, b"eglDestroySurface\0")?;
            let destroy_context: Destroy = symbol(&egl, b"eglDestroyContext\0")?;
            let proc: Proc = symbol(&egl, b"eglGetProcAddress\0")?;
            let read: ReadPixels = symbol(&gles, b"glReadPixels\0")?;
            let bind: BindFramebuffer = symbol(&gles, b"glBindFramebuffer\0")?;
            let get: GetInteger = symbol(&gles, b"glGetIntegerv\0")?;
            let display = display(std::ptr::null_mut());
            if display.is_null()
                || init(display, std::ptr::null_mut(), std::ptr::null_mut()) == 0
                || bind_api(0x30A0) == 0
            {
                return Err("Cannot initialize N64 graphics".into());
            }
            let attrs = [
                0x3033,
                1,
                0x3040,
                4,
                0x3024,
                8,
                0x3023,
                8,
                0x3022,
                8,
                0x3025,
                if depth { 16 } else { 0 },
                0x3026,
                if stencil { 8 } else { 0 },
                NONE,
            ];
            let mut config = std::ptr::null_mut();
            let mut count = 0;
            if choose(display, attrs.as_ptr(), &mut config, 1, &mut count) == 0 || count == 0 {
                return Err("No GLES2 pbuffer configuration for N64".into());
            }
            let surface = create_surface(
                display,
                config,
                [0x3057, MAX_SIDE as i32, 0x3056, MAX_SIDE as i32, NONE].as_ptr(),
            );
            if surface.is_null() {
                return Err("Cannot create N64 drawing surface".into());
            }
            let context = create_context(
                display,
                config,
                std::ptr::null_mut(),
                [0x3098, 2, NONE].as_ptr(),
            );
            if context.is_null() {
                destroy_surface(display, surface);
                return Err("Cannot create N64 graphics context".into());
            }
            if current(display, surface, surface, context) == 0 {
                destroy_context(display, context);
                destroy_surface(display, surface);
                return Err("Cannot activate N64 graphics".into());
            }
            let mut hw = Self {
                delete_framebuffers: symbol(&gles, b"glDeleteFramebuffers\0")?,
                delete_textures: symbol(&gles, b"glDeleteTextures\0")?,
                delete_renderbuffers: symbol(&gles, b"glDeleteRenderbuffers\0")?,
                _egl: egl,
                _gles: gles,
                display,
                surface,
                context,
                current,
                destroy_context,
                destroy_surface,
                proc,
                read,
                bind,
                get,
                framebuffer: 0,
                texture: 0,
                depthbuffer: 0,
            };
            hw.create_framebuffer(depth, stencil)?;
            Ok(hw)
        }
    }
    unsafe fn create_framebuffer(&mut self, depth: bool, stencil: bool) -> Result<(), String> {
        let gen_texture: unsafe extern "C" fn(i32, *mut u32) =
            symbol(&self._gles, b"glGenTextures\0")?;
        let bind_texture: unsafe extern "C" fn(u32, u32) = symbol(&self._gles, b"glBindTexture\0")?;
        let parameter: unsafe extern "C" fn(u32, u32, i32) =
            symbol(&self._gles, b"glTexParameteri\0")?;
        let tex_image: unsafe extern "C" fn(u32, i32, i32, i32, i32, i32, u32, u32, *const c_void) =
            symbol(&self._gles, b"glTexImage2D\0")?;
        let gen_fbo: unsafe extern "C" fn(i32, *mut u32) =
            symbol(&self._gles, b"glGenFramebuffers\0")?;
        let attach: unsafe extern "C" fn(u32, u32, u32, u32, i32) =
            symbol(&self._gles, b"glFramebufferTexture2D\0")?;
        let check: unsafe extern "C" fn(u32) -> u32 =
            symbol(&self._gles, b"glCheckFramebufferStatus\0")?;
        gen_texture(1, &mut self.texture);
        bind_texture(0x0DE1, self.texture);
        for key in [0x2800, 0x2801] {
            parameter(0x0DE1, key, 0x2600);
        }
        for key in [0x2802, 0x2803] {
            parameter(0x0DE1, key, 0x812F);
        }
        tex_image(
            0x0DE1,
            0,
            0x1908,
            MAX_SIDE as i32,
            MAX_SIDE as i32,
            0,
            0x1908,
            0x1401,
            std::ptr::null(),
        );
        gen_fbo(1, &mut self.framebuffer);
        (self.bind)(0x8D40, self.framebuffer);
        attach(0x8D40, 0x8CE0, 0x0DE1, self.texture, 0);
        if depth || stencil {
            let gen: unsafe extern "C" fn(i32, *mut u32) =
                symbol(&self._gles, b"glGenRenderbuffers\0")?;
            let bind: unsafe extern "C" fn(u32, u32) =
                symbol(&self._gles, b"glBindRenderbuffer\0")?;
            let storage: unsafe extern "C" fn(u32, u32, i32, i32) =
                symbol(&self._gles, b"glRenderbufferStorage\0")?;
            let attach: unsafe extern "C" fn(u32, u32, u32, u32) =
                symbol(&self._gles, b"glFramebufferRenderbuffer\0")?;
            gen(1, &mut self.depthbuffer);
            bind(0x8D41, self.depthbuffer);
            storage(
                0x8D41,
                if stencil { 0x88F0 } else { 0x81A5 },
                MAX_SIDE as i32,
                MAX_SIDE as i32,
            );
            attach(0x8D40, 0x8D00, 0x8D41, self.depthbuffer);
            if stencil {
                attach(0x8D40, 0x8D20, 0x8D41, self.depthbuffer);
            }
        }
        if check(0x8D40) != 0x8CD5 {
            return Err("Cannot create N64 framebuffer".into());
        }
        Ok(())
    }
    pub fn framebuffer(&self) -> usize {
        self.framebuffer as usize
    }
    pub unsafe fn proc_address(&self, name: *const c_char) -> *const c_void {
        (self.proc)(name)
    }
    pub fn frame(&self, width: u32, height: u32, bottom_left: bool) -> Option<Vec<u8>> {
        if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
            return None;
        }
        let mut rgba = vec![0; (width * height * 4) as usize];
        unsafe {
            let mut old = 0;
            (self.get)(0x8CA6, &mut old);
            (self.bind)(0x8D40, self.framebuffer);
            (self.read)(
                0,
                0,
                width as i32,
                height as i32,
                0x1908,
                0x1401,
                rgba.as_mut_ptr().cast(),
            );
            (self.bind)(0x8D40, old as u32);
        }
        let mut xrgb = vec![0; rgba.len()];
        for y in 0..height as usize {
            let sy = if bottom_left {
                height as usize - 1 - y
            } else {
                y
            };
            for x in 0..width as usize {
                let a = (sy * width as usize + x) * 4;
                let b = (y * width as usize + x) * 4;
                xrgb[b..b + 4].copy_from_slice(&[rgba[a + 2], rgba[a + 1], rgba[a], 0]);
            }
        }
        Some(xrgb)
    }
}
impl Drop for Hardware {
    fn drop(&mut self) {
        unsafe {
            (self.delete_framebuffers)(1, &self.framebuffer);
            (self.delete_textures)(1, &self.texture);
            (self.delete_renderbuffers)(1, &self.depthbuffer);
            (self.current)(
                self.display,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            (self.destroy_context)(self.display, self.context);
            (self.destroy_surface)(self.display, self.surface);
        }
        // Do not eglTerminate: the frontend uses the same display on its own thread.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires EGL/GLES2 (EGL_PLATFORM=surfaceless on Mesa)"]
    fn reads_gles_pixels_in_bgr_order_and_preserves_origin() {
        let hw = Hardware::open(true, false).unwrap();
        unsafe {
            let clear: unsafe extern "C" fn(u32) = symbol(&hw._gles, b"glClear\0").unwrap();
            let colour: unsafe extern "C" fn(f32, f32, f32, f32) =
                symbol(&hw._gles, b"glClearColor\0").unwrap();
            let scissor: unsafe extern "C" fn(i32, i32, i32, i32) =
                symbol(&hw._gles, b"glScissor\0").unwrap();
            let enable: unsafe extern "C" fn(u32) = symbol(&hw._gles, b"glEnable\0").unwrap();
            colour(1.0, 0.0, 0.0, 1.0);
            clear(0x4000);
            enable(0x0C11);
            scissor(0, 0, 2, 1);
            colour(0.0, 0.0, 1.0, 1.0);
            clear(0x4000);
        }
        assert_eq!(
            hw.frame(2, 2, true).unwrap(),
            [0, 0, 255, 0, 0, 0, 255, 0, 255, 0, 0, 0, 255, 0, 0, 0]
        );
        assert!(hw.frame(0, 2, true).is_none());
        assert!(hw.frame(641, 2, true).is_none());
    }
}
