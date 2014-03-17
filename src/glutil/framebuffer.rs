use std::vec_ng::Vec;
use std::cast;
use hgl::texture::{Texture, ImageInfo};
use hgl::texture;
use hgl::texture::pixel;
use gl;
use gl::types::{GLuint};

pub struct Framebuffer {
    width: uint,
    height: uint,
    framebuffer: GLuint,
    depthbuffer: GLuint,
    texture: ~Texture,
}

impl Framebuffer {
    pub fn new(width: uint, height: uint) -> Framebuffer {
        let info = ImageInfo::new()
            .width(width as i32)
            .height(height as i32)
            .pixel_format(texture::pixel::RGBA)
            .pixel_type(pixel::UNSIGNED_BYTE)
            ;
        let pixels = Vec::from_elem(width * height * 4, 0u8);
        let texture = Texture::new(texture::Texture2D, info, pixels.get(0));
        texture.filter(texture::Nearest);
        texture.wrap(texture::ClampToEdge);

        let mut fb: GLuint = 0;
        unsafe { gl::GenFramebuffers(1, &mut fb); }

        // Make a depth buffer.
        let mut db: GLuint = 0;
        unsafe { gl::GenRenderbuffers(1, &mut db); }

        let ret = Framebuffer {
            width: width,
            height: height,
            framebuffer: fb,
            depthbuffer: db,
            texture: ~texture,
        };

        //ret.bind();
        gl::BindFramebuffer(gl::FRAMEBUFFER, ret.framebuffer);
        gl::BindRenderbuffer(gl::RENDERBUFFER, ret.depthbuffer);
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D, ret.texture.name, 0);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, width as i32, height as i32);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT,
            gl::RENDERBUFFER, ret.depthbuffer);

        assert!(
            gl::CheckFramebufferStatus(gl::FRAMEBUFFER) ==
            gl::FRAMEBUFFER_COMPLETE);

        ret.unbind();

        ret
    }

    pub fn bind(&self) {
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        gl::BindRenderbuffer(gl::RENDERBUFFER, self.depthbuffer);
    }

    pub fn unbind(&self) {
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let ret = Vec::from_elem(self.width * self.height * 4, 0u8);
        self.texture.bind();
        unsafe {
            gl::GetTexImage(
                gl::TEXTURE_2D, 0, gl::RGBA, gl::UNSIGNED_BYTE,
                cast::transmute(ret.get(0)));
        }
        gl::BindTexture(gl::TEXTURE_2D, 0);
        ret
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.unbind();
            gl::DeleteFramebuffers(1, &self.framebuffer);
            gl::DeleteRenderbuffers(1, &self.depthbuffer);
        }
    }
}
