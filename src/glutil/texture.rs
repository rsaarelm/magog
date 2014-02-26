use std::cast;
use std::vec;
use gl;
use gl::types::{GLuint, GLint, GLenum};

pub struct Texture {
    priv id: GLuint,
    priv width: uint,
    priv height: uint,
    priv bpp: uint,
}

impl Texture {
    fn new_data(width: uint, height: uint, data: Option<&[u8]>, format: GLenum) -> Texture {
        let mut ret = Texture::new();

        ret.bind();
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
        match data {
            Some(d) => unsafe {
                gl::TexImage2D(
                    gl::TEXTURE_2D, 0, format as GLint,
                    width as GLint, height as GLint, 0,
                    format, gl::UNSIGNED_BYTE, cast::transmute(&d[0]));
            },
            _ => ()
        }

        ret.width = width;
        ret.height = height;
        if format == gl::ALPHA {
            ret.bpp = 1;
        }
        else if format == gl::RGBA {
            ret.bpp = 4;
        } else {
            fail!("Unknown format");
        }

        ret.unbind();
        ret
    }

    pub fn new_rgba(width: uint, height: uint, data: Option<&[u8]>) -> Texture {
        match data {
            Some(data) => assert!(data.len() == width * height * 4),
            _ => ()
        };
        Texture::new_data(width, height, data, gl::RGBA)
    }

    pub fn new_alpha(width: uint, height: uint, data: Option<&[u8]>) -> Texture {
        match data {
            Some(data) => assert!(data.len() == width * height),
            _ => ()
        };
        Texture::new_data(width, height, data, gl::ALPHA)
    }

    pub fn new() -> Texture {
        let mut id: GLuint = 0;
        unsafe { gl::GenTextures(1, &mut id); }
        Texture {
            id: id,
            width: 0,
            height: 0,
            bpp: 1,
        }
    }

    pub fn bind(&self) {
        gl::BindTexture(gl::TEXTURE_2D, self.id);
    }

    pub fn unbind(&self) {
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    /// Creates a framebuffer for this texture, executes f rendering
    /// into it instead of the the screen.
    pub fn render_to(&self, f: ||) {
        let mut fb: GLuint = 0;
        unsafe { gl::GenFramebuffers(1, &mut fb); }

        gl::BindFramebuffer(gl::FRAMEBUFFER, fb);
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D, self.id, 0);
        assert!(
            gl::CheckFramebufferStatus(gl::FRAMEBUFFER) ==
            gl::FRAMEBUFFER_COMPLETE);

        f();

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        unsafe {
            gl::DeleteFramebuffers(1, &fb);
        }
    }

    pub fn get_bytes(&self) -> ~[u8] {
        let mut ret = vec::from_elem(self.width * self.height * 4, 0u8);
        self.bind();
        unsafe {
            gl::GetTexImage(
                gl::TEXTURE_2D, 0, gl::RGBA, gl::UNSIGNED_BYTE,
                cast::transmute(&mut ret[0]));
        }
        ret
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
