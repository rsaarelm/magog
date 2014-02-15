use opengles::gl2;
use opengles::gl2::{GLuint, GLint, GLenum};

pub struct Texture {
    priv id: GLuint,
}

impl Texture {
    fn new_data(width: uint, height: uint, data: Option<&[u8]>, format: GLenum) -> Texture {
        let ret = Texture::new();

        ret.bind();
        gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_MIN_FILTER, gl2::NEAREST as gl2::GLint);
        gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_MAG_FILTER, gl2::NEAREST as gl2::GLint);
        gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_WRAP_S, gl2::CLAMP_TO_EDGE as gl2::GLint);
        gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_WRAP_T, gl2::CLAMP_TO_EDGE as gl2::GLint);
        gl2::tex_image_2d(
                gl2::TEXTURE_2D, 0, format as GLint,
                width as gl2::GLint, height as gl2::GLint, 0,
                format, gl2::UNSIGNED_BYTE, data);

        ret.unbind();
        ret
    }

    pub fn new_rgba(width: uint, height: uint, data: Option<&[u8]>) -> Texture {
        match data {
            Some(data) => assert!(data.len() == width * height * 4),
            _ => ()
        };
        Texture::new_data(width, height, data, gl2::RGBA)
    }

    pub fn new_alpha(width: uint, height: uint, data: Option<&[u8]>) -> Texture {
        match data {
            Some(data) => assert!(data.len() == width * height),
            _ => ()
        };
        Texture::new_data(width, height, data, gl2::ALPHA)
    }

    pub fn new() -> Texture {
        let ids = gl2::gen_textures(1);
        Texture{ id: ids[0] }
    }

    pub fn bind(&self) {
        gl2::bind_texture(gl2::TEXTURE_2D, self.id);
    }

    pub fn unbind(&self) {
        gl2::bind_texture(gl2::TEXTURE_2D, 0);
    }

    /// Creates a framebuffer for this texture, executes f rendering
    /// into it instead of the the screen.
    pub fn render_to(&self, f: ||) {
        let fb = gl2::gen_framebuffers(1)[0];

        gl2::bind_framebuffer(gl2::FRAMEBUFFER, fb);
        gl2::framebuffer_texture_2d(
            gl2::FRAMEBUFFER, gl2::COLOR_ATTACHMENT0,
            gl2::TEXTURE_2D, self.id, 0);
        assert!(
            gl2::check_framebuffer_status(gl2::FRAMEBUFFER) ==
            gl2::FRAMEBUFFER_COMPLETE);

        f();

        gl2::bind_framebuffer(gl2::FRAMEBUFFER, 0);
        gl2::delete_frame_buffers([fb]);
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        gl2::delete_textures(&[self.id]);
    }
}
