use opengles::gl2;
use opengles::gl2::{GLint, GLuint};

pub struct Shader {
    priv program: GLuint,
}

// Will crash on failure, don't feed it untrusted shaders. (And don't support
// target platforms that can't handle your shader version, I guess...)
fn compile_shader(src: &str, kind: GLuint) -> GLuint {
    let shader = gl2::create_shader(kind);
    assert!(shader != 0);
    gl2::shader_source(shader, [src.as_bytes()]);
    gl2::compile_shader(shader);

    let result = gl2::get_shader_iv(shader, gl2::COMPILE_STATUS);
    if result == 0 {
        let err = gl2::get_shader_info_log(shader);
        gl2::delete_shader(shader);
        fail!(err);
    }
    shader
}

fn link_program(vshader: GLuint, fshader: GLuint) -> GLuint {
    let prog = gl2::create_program();
    gl2::attach_shader(prog, vshader);
    gl2::attach_shader(prog, fshader);
    gl2::link_program(prog);

    let result = gl2::get_program_iv(prog, gl2::LINK_STATUS);
    if result == 0 {
        let err = gl2::get_program_info_log(prog);
        gl2::delete_program(prog);
        fail!(err);
    }
    prog
}

impl Shader {
    pub fn new(vert_src: &str, frag_src: &str) -> Shader {
        let vshader = compile_shader(vert_src, gl2::VERTEX_SHADER);
        let fshader = compile_shader(frag_src, gl2::FRAGMENT_SHADER);
        let prog = link_program(vshader, fshader);
        gl2::delete_shader(vshader);
        gl2::delete_shader(fshader);
        Shader { program: prog }
    }

    pub fn bind(&self) {
        gl2::use_program(self.program);
    }

    pub fn attrib(&self, name: &str) -> GLuint {
        let result = gl2::get_attrib_location(self.program, name);
        if result < 0 {
            fail!("Attrib {} not found", name);
        }
        result as GLuint
    }

    pub fn uniform(&self, name: &str) -> GLint {
        let result = gl2::get_uniform_location(self.program, name);
        if result < 0 {
            fail!("Uniform {} not found", name);
        }
        result
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        gl2::delete_program(self.program);
    }
}
