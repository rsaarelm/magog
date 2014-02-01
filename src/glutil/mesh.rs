use opengles::gl2;
use opengles::gl2::{GLuint};
use cgmath::point::{Point2, Point3};
use cgmath::aabb::{Aabb, Aabb2};
use std::mem;

use gl_check;

use shader::Shader;

pub struct Mesh {
    priv vertices: GLuint,
    priv uvs: GLuint,
    priv indices: GLuint,
    priv num_indices: i32,
}

impl Mesh {
    pub fn new(vertices: ~[Point3<f32>], uvs: ~[Point2<f32>], indices: ~[u32]) -> Mesh {
	let gen = gl_check!(gl2::gen_buffers(3));
	let vert = gen[0];
	let uv = gen[1];
	let ind = gen[2];

	gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, vert));
	gl_check!(gl2::buffer_data(gl2::ARRAY_BUFFER, vertices, gl2::STATIC_DRAW));

	gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, uv));
	gl_check!(gl2::buffer_data(gl2::ARRAY_BUFFER, uvs, gl2::STATIC_DRAW));

	gl_check!(gl2::bind_buffer(gl2::ELEMENT_ARRAY_BUFFER, ind));
	gl_check!(gl2::buffer_data(gl2::ELEMENT_ARRAY_BUFFER, indices, gl2::STATIC_DRAW));

	Mesh {
	    vertices: vert,
            uvs: uv,
	    indices: ind,
	    num_indices: indices.len() as i32,
	}
    }

    pub fn render(&self, shader: &Shader) {
        let in_pos = shader.attrib("in_pos").unwrap();
	gl_check!(gl2::enable_vertex_attrib_array(in_pos));
	gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, self.vertices));
	gl_check!(gl2::vertex_attrib_pointer_f32(in_pos, 3, false, mem::size_of::<Point3<f32>>() as i32, 0));
        let in_texcoord = shader.attrib("in_texcoord").unwrap();
	gl_check!(gl2::enable_vertex_attrib_array(in_texcoord));
	gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, self.uvs));
	gl_check!(gl2::vertex_attrib_pointer_f32(in_texcoord, 2, false, mem::size_of::<Point2<f32>>() as i32, 0));

	gl_check!(gl2::bind_buffer(gl2::ELEMENT_ARRAY_BUFFER, self.indices));
	gl_check!(gl2::draw_elements(gl2::TRIANGLES, self.num_indices, gl2::UNSIGNED_INT, None));
	gl_check!(gl2::bind_buffer(gl2::ELEMENT_ARRAY_BUFFER, 0));
	gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, 0));

	gl_check!(gl2::disable_vertex_attrib_array(in_texcoord));
	gl_check!(gl2::disable_vertex_attrib_array(in_pos));
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
	gl_check!(gl2::delete_buffers(&[self.vertices, self.indices]));
    }
}

pub fn draw_texture_rect(shader: &Shader, area: &Aabb2<f32>, texcoord: &Aabb2<f32>) {
   let mesh = Mesh::new(
       ~[Point3::new(area.min().x, area.min().y, 0.0f32),
         Point3::new(area.max().x, area.min().y, 0.0f32),
         Point3::new(area.min().x, area.max().y, 0.0f32),
         Point3::new(area.max().x, area.max().y, 0.0f32),
       ],
       ~[Point2::new(texcoord.min().x, texcoord.max().y),
         Point2::new(texcoord.max().x, texcoord.max().y),
         Point2::new(texcoord.min().x, texcoord.min().y),
         Point2::new(texcoord.max().x, texcoord.min().y),
       ],
       ~[0, 1, 3, 0, 2, 3]);
   mesh.render(shader);
}
