use opengles::gl2;
use opengles::gl2::{GLint, GLuint};

use gl_check;

pub struct Shader {
    priv vshader: GLuint,
    priv fshader: GLuint,
    priv program: GLuint,
}

// Will crash on failure, don't feed it untrusted shaders. (And don't support
// target platforms that can't handle your shader version, I guess...)
fn compile_shader(src: &str, kind: GLuint) -> Result<GLuint, ~str> {
    let shader = gl_check!(gl2::create_shader(kind));
    assert!(shader != 0);
    gl_check!(gl2::shader_source(shader, [src.as_bytes().to_owned()]));
    gl_check!(gl2::compile_shader(shader));

    let result = gl_check!(gl2::get_shader_iv(shader, gl2::COMPILE_STATUS));
    if result == 0 {
        let err = gl_check!(gl2::get_shader_info_log(shader));
        gl_check!(gl2::delete_shader(shader));
        Err(err)
    } else {
        Ok(shader)
    }
}

fn link_program(vshader: GLuint, fshader: GLuint) -> Result<GLuint, ~str> {
    let prog = gl_check!(gl2::create_program());
    gl_check!(gl2::attach_shader(prog, vshader));
    gl_check!(gl2::attach_shader(prog, fshader));
    gl_check!(gl2::link_program(prog));

    let result = gl_check!(gl2::get_program_iv(prog, gl2::LINK_STATUS));
    if result == 0 {
        let err = gl_check!(gl2::get_program_info_log(prog));
        gl_check!(gl2::delete_program(prog));
        Err(err)
    } else {
        Ok(prog)
    }
}

impl Shader {
    pub fn new(vert_src: &str, frag_src: &str) -> Shader {
        let vshader = match compile_shader(vert_src, gl2::VERTEX_SHADER) {
            Ok(s) => s,
            Err(e) => fail!(e)
        };
        let fshader = match compile_shader(frag_src, gl2::FRAGMENT_SHADER) {
            Ok(s) => s,
            Err(e) => fail!(e)
        };
        Shader{
            vshader: vshader,
            fshader: fshader,
            program: match link_program(vshader, fshader) {
                Ok(p) => p,
                Err(e) => fail!(e)
            },
        }
    }

    pub fn bind(&self) {
        gl_check!(gl2::use_program(self.program));
    }

    pub fn attrib(&self, name: &str) -> Option<GLuint> {
        let result = gl_check!(gl2::get_attrib_location(self.program, name));
        if result < 0 {
            return None;
        }
        Some(result as GLuint)
    }

    pub fn uniform(&self, name: &str) -> Option<GLint> {
        let result = gl_check!(gl2::get_uniform_location(self.program, name));
        if result < 0 {
            return None;
        }
        Some(result as GLint)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        gl_check!(gl2::delete_program(self.program));
        gl_check!(gl2::delete_shader(self.vshader));
        gl_check!(gl2::delete_shader(self.fshader));
    }
}
