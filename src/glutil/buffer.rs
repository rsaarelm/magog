use opengles::gl2;
use opengles::gl2::{GLuint, GLenum};

pub enum BufferDataType {
    /// Create once, draw many times.
    StaticDraw,
    /// Create once, draw once.
    StreamDraw,
    /// Create and modify later, draw many times.
    DynamicDraw,
}

pub struct Buffer {
    priv id: GLuint,
    priv gl_t: GLenum,
}

impl Buffer {
    fn new(gl_t: GLenum) -> Buffer {
        let gen = gl2::gen_buffers(1);
        assert!(gen.len() == 1);
        Buffer {
            id: gen[0],
            gl_t: gl_t,
        }
    }

    pub fn new_array() -> Buffer { Buffer::new(gl2::ARRAY_BUFFER) }
    pub fn new_element_array() -> Buffer { Buffer::new(gl2::ELEMENT_ARRAY_BUFFER) }

    pub fn bind(&self) {
        gl2::bind_buffer(self.gl_t, self.id);
    }

    pub fn load_data<S>(&self, data: &[S], dt: BufferDataType) {
        let gl_dt = match dt {
            StaticDraw => gl2::STATIC_DRAW,
            StreamDraw => gl2::STREAM_DRAW,
            DynamicDraw => gl2::DYNAMIC_DRAW,
        };
        self.bind();
        gl2::buffer_data(self.gl_t, data, gl_dt);
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        gl2::delete_buffers([self.id]);
    }
}
