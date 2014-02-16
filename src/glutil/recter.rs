use std::cmp::min;
use opengles::gl2;
use cgmath::point::{Point2, Point3};
use cgVector = cgmath::vector::Vector;
use cgmath::vector::{Vec2};
use cgmath::aabb::{Aabb, Aabb2};
use shader::Shader;
use app::Color;
use buffer;
use buffer::Buffer;

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

    pub fn render(&mut self, shader: &Shader) {
        if self.vertices.len() == 0 {
            return;
        }
        let vert = Buffer::new_array();
        let tex = Buffer::new_array();
        let col = Buffer::new_array();

        // Bind shader vars.
        let in_pos = shader.attrib("in_pos");
        vert.bind();
        vert.load_data(self.vertices, buffer::StreamDraw);
        gl_check!(gl2::vertex_attrib_pointer_f32(in_pos, 3, false, 0, 0));
        gl_check!(gl2::enable_vertex_attrib_array(in_pos));
        let in_texcoord = shader.attrib("in_texcoord");
        tex.bind();
        tex.load_data(self.texcoords, buffer::StreamDraw);
        gl_check!(gl2::vertex_attrib_pointer_f32(in_texcoord, 2, false, 0, 0));
        gl_check!(gl2::enable_vertex_attrib_array(in_texcoord));
        let in_color = shader.attrib("in_color");
        col.bind();
        col.load_data(self.colors, buffer::StreamDraw);
        gl_check!(gl2::vertex_attrib_pointer_f32(in_color, 4, false, 0, 0));
        gl_check!(gl2::enable_vertex_attrib_array(in_color));

        // Draw!
        gl_check!(gl2::draw_arrays(gl2::TRIANGLES, 0, self.vertices.len() as i32));
        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, 0));

        gl_check!(gl2::disable_vertex_attrib_array(in_color));
        gl_check!(gl2::disable_vertex_attrib_array(in_texcoord));
        gl_check!(gl2::disable_vertex_attrib_array(in_pos));

        self.clear();
    }
}

pub fn screen_bound(dim: &Vec2<f32>, area: &Vec2<f32>) -> Aabb2<f32> {
    let mut scale = min(
        area.x / dim.x,
        area.y / dim.y);
    if scale > 1.0 {
        scale = scale.floor();
    }

    let dim = Point2::new(dim.x * 2f32 * scale / area.x, dim.y * 2f32 * scale / area.y);
    let bound = Aabb2::new(Point2::new(0f32, 0f32), dim);
    bound.add_v(&Vec2::new(-dim.x / 2f32, -dim.y / 2f32))
}

pub fn draw_screen_texture(bound: &Aabb2<f32>, shader: &Shader) {
    let vertices = ~[
        Point2::new(bound.min.x, bound.min.y),
        Point2::new(bound.max.x, bound.min.y),
        Point2::new(bound.min.x, bound.max.y),

        Point2::new(bound.max.x, bound.min.y),
        Point2::new(bound.max.x, bound.max.y),
        Point2::new(bound.min.x, bound.max.y),
    ];

    let texcoords = ~[
        Point2::new(0f32, 1f32),
        Point2::new(1f32, 1f32),
        Point2::new(0f32, 0f32),

        Point2::new(1f32, 1f32),
        Point2::new(1f32, 0f32),
        Point2::new(0f32, 0f32),
    ];

    let vert = Buffer::new_array();
    let tex = Buffer::new_array();

    let in_pos = shader.attrib("in_pos");
    vert.bind();
    vert.load_data(vertices, buffer::StreamDraw);
    gl_check!(gl2::vertex_attrib_pointer_f32(in_pos, 2, false, 0, 0));
    gl_check!(gl2::enable_vertex_attrib_array(in_pos));
    let in_texcoord = shader.attrib("in_texcoord");
    tex.bind();
    tex.load_data(texcoords, buffer::StreamDraw);
    gl_check!(gl2::vertex_attrib_pointer_f32(in_texcoord, 2, false, 0, 0));
    gl_check!(gl2::enable_vertex_attrib_array(in_texcoord));

    // Draw!
    gl_check!(gl2::draw_arrays(gl2::TRIANGLES, 0, vertices.len() as i32));
    gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, 0));

    gl_check!(gl2::disable_vertex_attrib_array(in_texcoord));
    gl_check!(gl2::disable_vertex_attrib_array(in_pos));
}
