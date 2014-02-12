use opengles::gl2;
use cgmath::vector::{Vec2};
use cgmath::point::{Point2, Point3};
use cgmath::aabb::{Aabb, Aabb2};
use shader::Shader;
use app::Color;

use gl_check;

pub struct Recter {
    vertices: ~[Point3<f32>],
    texcoords: ~[Point2<f32>],
    colors: ~[Color],
}

impl Recter {
    pub fn new() -> Recter {
        Recter {
            vertices: ~[],
            texcoords: ~[],
            colors: ~[],
        }
    }

    pub fn add(&mut self, area: &Aabb2<f32>, texcoords: &Aabb2<f32>, color: &Color) {
        self.vertices.push(Point3::new(area.min().x, area.min().y, 0.0));
        self.texcoords.push(Point2::new(texcoords.min().x, texcoords.min().y));
        self.colors.push(*color);

        self.vertices.push(Point3::new(area.min().x, area.max().y, 0.0));
        self.texcoords.push(Point2::new(texcoords.min().x, texcoords.max().y));
        self.colors.push(*color);

        self.vertices.push(Point3::new(area.max().x, area.max().y, 0.0));
        self.texcoords.push(Point2::new(texcoords.max().x, texcoords.max().y));
        self.colors.push(*color);


        self.vertices.push(Point3::new(area.min().x, area.min().y, 0.0));
        self.texcoords.push(Point2::new(texcoords.min().x, texcoords.min().y));
        self.colors.push(*color);

        self.vertices.push(Point3::new(area.max().x, area.max().y, 0.0));
        self.texcoords.push(Point2::new(texcoords.max().x, texcoords.max().y));
        self.colors.push(*color);

        self.vertices.push(Point3::new(area.max().x, area.min().y, 0.0));
        self.texcoords.push(Point2::new(texcoords.max().x, texcoords.min().y));
        self.colors.push(*color);

    }

    pub fn clear(&mut self) {
        self.vertices = ~[];
        self.texcoords = ~[];
        self.colors = ~[];
    }

    pub fn render(
        &mut self,
        shader: &Shader, scale: &Vec2<f32>,
        offset: &Vec2<f32>) {
        if self.vertices.len() == 0 {
            return;
        }
        // Generate buffers.
        // TODO: Wrap STREAM_DRAW buffers into RAII handles.
        let gen = gl_check!(gl2::gen_buffers(3));
        let vert = gen[0];
        let tex = gen[1];
        let col = gen[2];

        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, vert));
        gl_check!(gl2::buffer_data(gl2::ARRAY_BUFFER, self.vertices, gl2::STREAM_DRAW));

        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, tex));
        gl_check!(gl2::buffer_data(gl2::ARRAY_BUFFER, self.texcoords, gl2::STREAM_DRAW));

        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, col));
        gl_check!(gl2::buffer_data(gl2::ARRAY_BUFFER, self.colors, gl2::STREAM_DRAW));

        // Bind shader vars.
        let in_pos = shader.attrib("in_pos").unwrap();
        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, vert));
        gl_check!(gl2::vertex_attrib_pointer_f32(in_pos, 3, false, 0, 0));
        gl_check!(gl2::enable_vertex_attrib_array(in_pos));
        let in_texcoord = shader.attrib("in_texcoord").unwrap();
        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, tex));
        gl_check!(gl2::vertex_attrib_pointer_f32(in_texcoord, 2, false, 0, 0));
        gl_check!(gl2::enable_vertex_attrib_array(in_texcoord));
        let in_color = shader.attrib("in_color").unwrap();
        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, col));
        gl_check!(gl2::vertex_attrib_pointer_f32(in_color, 4, false, 0, 0));
        gl_check!(gl2::enable_vertex_attrib_array(in_color));

        let x = 2f32 / scale.x;
        let y = -2f32 / scale.y;
        let dx = -1f32 + 2.0 * offset.x / scale.x;
        let dy = 1f32 - 2.0 * offset.y / scale.y;
        let transform = &[
            x,    0f32, 0f32, 0f32,
            0f32, y,    0f32, 0f32,
            0f32, 0f32, 1f32, 0f32,
            dx,   dy,   0f32, 1f32,
            ];
        gl_check!(gl2::uniform_matrix_4fv(
                shader.uniform("transform").unwrap(),
                false,
                transform));

        // Draw!
        gl_check!(gl2::draw_arrays(gl2::TRIANGLES, 0, self.vertices.len() as i32));
        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, 0));

        gl_check!(gl2::disable_vertex_attrib_array(in_color));
        gl_check!(gl2::disable_vertex_attrib_array(in_texcoord));
        gl_check!(gl2::disable_vertex_attrib_array(in_pos));

        self.clear();
        gl2::delete_buffers(gen);
    }
}

