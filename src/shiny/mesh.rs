use opengles::gl2;
use opengles::gl2::{GLuint};
use cgmath::point::Point3;
use std::mem;

use gl_check;

pub struct Mesh {
    priv vertices: GLuint,
    priv indices: GLuint,
    priv num_indices: i32,
}

impl Mesh {
    pub fn new(vertices: ~[Point3<f32>], indices: ~[u32]) -> Mesh {
	let gen = gl_check!(gl2::gen_buffers(2));
	let vert = gen[0];
	let ind = gen[1];

	gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, vert));
	gl_check!(gl2::buffer_data(gl2::ARRAY_BUFFER, vertices, gl2::STATIC_DRAW));

	gl_check!(gl2::bind_buffer(gl2::ELEMENT_ARRAY_BUFFER, ind));
	gl_check!(gl2::buffer_data(gl2::ELEMENT_ARRAY_BUFFER, indices, gl2::STATIC_DRAW));

	Mesh {
	    vertices: vert,
	    indices: ind,
	    num_indices: indices.len() as i32,
	}
    }

    pub fn render(&self) {
	gl_check!(gl2::enable_vertex_attrib_array(0));
	gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, self.vertices));
	gl_check!(gl2::vertex_attrib_pointer_f32(0, 3, false, mem::size_of::<Point3<f32>>() as i32, 0));
	gl_check!(gl2::bind_buffer(gl2::ELEMENT_ARRAY_BUFFER, self.indices));
	gl_check!(gl2::draw_elements(gl2::TRIANGLES, self.num_indices, gl2::UNSIGNED_INT, None));
	gl_check!(gl2::disable_vertex_attrib_array(0));
	gl_check!(gl2::bind_buffer(gl2::ELEMENT_ARRAY_BUFFER, 0));
	gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, 0));
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
	gl_check!(gl2::delete_buffers(&[self.vertices, self.indices]));
    }
}
