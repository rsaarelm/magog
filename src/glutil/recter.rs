use std::cmp::min;
use std::mem::size_of;
use cgmath::point::{Point2};
use cgVector = cgmath::vector::Vector;
use cgmath::vector::{Vec2};
use cgmath::aabb::{Aabb, Aabb2};
use gl;
use hgl;
use hgl::{Program, Vao, Vbo};
use hgl::buffer;
use color::{RGB, ToRGB};

struct Vertex {
    px: f32,
    py: f32,
    pz: f32,

    u: f32,
    v: f32,

    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Vertex {
    pub fn new(
        px: f32, py: f32, pz: f32,
        u: f32, v: f32,
        color: &RGB<f32>, a: f32) -> Vertex {
        Vertex {
            px: px,
            py: py,
            pz: pz,
            u: u,
            v: v,
            r: color.r,
            g: color.g,
            b: color.b,
            a: a
        }
    }

    pub fn stride() -> i32 { size_of::<Vertex>() as i32 }
    pub fn pos_offset() -> uint { 0 }
    pub fn tex_offset() -> uint { 3 * size_of::<f32>() }
    pub fn color_offset() -> uint { 5 * size_of::<f32>() }
}

pub struct Recter {
    priv vertices: ~[Vertex],
}

impl Recter {
    pub fn new() -> Recter {
        Recter {
            vertices: ~[],
        }
    }

    pub fn add<C: ToRGB>(
        &mut self, area: &Aabb2<f32>, z: f32, texcoords: &Aabb2<f32>, color: &C, alpha: f32) {
        let c = color.to_rgb::<f32>();

        self.vertices.push(Vertex::new(
                area.min().x, area.min().y, z,
                texcoords.min().x, texcoords.min().y,
                &c, alpha));

        self.vertices.push(Vertex::new(
                area.min().x, area.max().y, z,
                texcoords.min().x, texcoords.max().y,
                &c, alpha));

        self.vertices.push(Vertex::new(
                area.max().x, area.max().y, z,
                texcoords.max().x, texcoords.max().y,
                &c, alpha));


        self.vertices.push(Vertex::new(
                area.min().x, area.min().y, z,
                texcoords.min().x, texcoords.min().y,
                &c, alpha));

        self.vertices.push(Vertex::new(
                area.max().x, area.max().y, z,
                texcoords.max().x, texcoords.max().y,
                &c, alpha));

        self.vertices.push(Vertex::new(
                area.max().x, area.min().y, z,
                texcoords.max().x, texcoords.min().y,
                &c, alpha));
    }

    pub fn clear(&mut self) {
        self.vertices = ~[];
    }

    pub fn render(&self, program: &Program) {
        if self.vertices.len() == 0 {
            return;
        }
        let vao = Vao::new();
        let vbo = Vbo::from_data(self.vertices, buffer::StreamDraw);

        program.bind();
        vao.bind();
        vbo.bind();

        vao.enable_attrib(
            program, "in_pos", gl::FLOAT, 3,
            Vertex::stride(), Vertex::pos_offset());
        vao.enable_attrib(
            program, "in_texcoord", gl::FLOAT, 2,
            Vertex::stride(), Vertex::tex_offset());
        vao.enable_attrib(
            program, "in_color", gl::FLOAT, 4,
            Vertex::stride(), Vertex::color_offset());

        vao.draw_array(hgl::Triangles, 0, self.vertices.len() as i32);
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
