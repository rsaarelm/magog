use opengles::gl2;
use opengles::gl2::{GLuint, GLint, GLenum};

use gl_check;

pub struct Texture {
    priv id: GLuint,
}

impl Texture {
    fn new(width: uint, height: uint, data: &[u8], format: GLenum) -> Texture {
        let ids = gl_check!(gl2::gen_textures(1));
        let ret = Texture{ id: ids[0] };

        ret.bind();
        gl_check!(gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_MIN_FILTER, gl2::LINEAR as gl2::GLint));
        gl_check!(gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_MAG_FILTER, gl2::LINEAR as gl2::GLint));
        gl_check!(gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_WRAP_S, gl2::CLAMP_TO_EDGE as gl2::GLint));
        gl_check!(gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_WRAP_T, gl2::CLAMP_TO_EDGE as gl2::GLint));
        gl_check!(gl2::tex_image_2d(
                gl2::TEXTURE_2D, 0, format as GLint,
                width as gl2::GLint, height as gl2::GLint, 0,
                format, gl2::UNSIGNED_BYTE, Some(data)));

        ret.unbind();
        ret
    }

    pub fn new_rgba(width: uint, height: uint, data: &[u8]) -> Texture {
        assert!(data.len() == width * height * 4);
        Texture::new(width, height, data, gl2::RGBA)
    }

    pub fn new_alpha(width: uint, height: uint, data: &[u8]) -> Texture {
        assert!(data.len() == width * height);
        Texture::new(width, height, data, gl2::ALPHA)
    }

    pub fn bind(&self) {
        gl_check!(gl2::bind_texture(gl2::TEXTURE_2D, self.id));
    }

    pub fn unbind(&self) {
        gl_check!(gl2::bind_texture(gl2::TEXTURE_2D, 0));
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
	gl_check!(gl2::delete_textures(&[self.id]));
    }
}
