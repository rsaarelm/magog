#[macro_use]
extern crate glium;
extern crate calx_system;
extern crate calx_window;
extern crate cgmath;

use std::default::Default;
use calx_window::{WindowBuilder, Event, Key};
use cgmath::Angle;
use glium::Surface;

/// Cube face.
#[derive(Copy, Clone)]
enum Face {
    East,
    North,
    Up,
    West,
    South,
    Down,
}

impl Face {
    /// Normal vector for the face.
    pub fn normal(&self) -> [f32; 3] {
        match *self {
            Face::East => [1.0, 0.0, 0.0],
            Face::North => [0.0, 1.0, 0.0],
            Face::Up => [0.0, 0.0, 1.0],
            Face::West => [-1.0, 0.0, 0.0],
            Face::South => [0.0, -1.0, 0.0],
            Face::Down => [0.0, 0.0, -1.0],
        }
    }

    /// Counter-clockwise vertex list using cube vertex indices.
    #[inline]
    pub fn vertices(&self) -> [usize; 4] {
        [[1, 3, 7, 5],
         [2, 6, 7, 3],
         [4, 5, 7, 6],
         [0, 4, 6, 2],
         [0, 1, 5, 4],
         [0, 2, 3, 1]][*self as usize]
    }
}

/// Vertex of a default [0, 0, 0] - [1, 1, 1] cube.
///
/// There are 8 indices, corresponding to 3 bits. The low bit denotes x-axis
/// extension, the middle bit y-axis extension and the high bit z-axis
/// extension.
///
/// ```notrust
///     6 --- 7
///    /:    /|
///   4 --- 5 |
///   | :   | |      Z
///   | 2 ..|.3      | Y
///   |.    |/       |/
///   0 --- 1        o---X
///
/// ```
#[inline]
fn cube_vertex(index: usize) -> [f32; 3] {
    [[0.0, 0.0, 0.0],
     [1.0, 0.0, 0.0],
     [0.0, 1.0, 0.0],
     [1.0, 1.0, 0.0],
     [0.0, 0.0, 1.0],
     [1.0, 0.0, 1.0],
     [0.0, 1.0, 1.0],
     [1.0, 1.0, 1.0]][index]
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}
implement_vertex!(Vertex, pos, normal, color);

struct Voxel {
    color: [f32; 4],
}

fn get_voxel(voxel_pos: [i32; 3]) -> Option<Voxel> {
    // TODO: Perlin noise landscape or something.
    if voxel_pos[2] < 0 {
        Some(Voxel { color: [0.0, 1.0, 0.0, 1.0] })
    } else {
        None
    }
}

fn main() {
    let mut window = WindowBuilder::new().set_title("Voxel demo").build();

    let shader = program!(&window.display,
            150 => {
            vertex: "
                #version 150 core

                uniform mat4 projection;
                uniform mat4 modelview;

                in vec3 pos;
                in vec3 normal;
                in vec4 color;
                out vec3 v_pos;
                out vec3 v_normal;
                out vec4 v_color;

                void main() {
                    v_pos = pos;
                    v_normal = normal;
                    v_color = color;
                    gl_Position = projection * modelview * vec4(v_pos, 1.0);
                }
            ",

            fragment: "
                #version 150 core

                in vec3 v_normal;
                in vec4 v_color;
                out vec4 f_color;
                const vec3 LIGHT = vec3(-0.2, 0.8, 0.1);

                void main() {
                    float lum = max(dot(normalize(v_normal), normalize(LIGHT)), 0.0);
                    f_color = (0.3 + 0.7 * lum) * v_color;
                }
            ",
            }
        ).unwrap();

    let mut tick = 0;
    loop {
        for e in window.events().into_iter() {
            match e {
                Event::Quit => return,
                Event::KeyPress(Key::Escape) => return,
                Event::KeyPress(Key::F12) => {
                    calx_system::save_screenshot("calx", window.screenshot())
                        .unwrap();
                }
                _ => (),
            }
        }
        window.clear(0x7799DDFF);


        // Camera setup

        let projection: cgmath::Matrix4<f32> =
            cgmath::PerspectiveFov {
                fovy: cgmath::Deg::new(45.0).into(),
                aspect: 640.0 / 380.0, // XXX: Hardcoding
                near: 0.1,
                far: 1024.0,
            }
            .into();

        let a = (tick as f32) / 32.0;
        let modelview: cgmath::Matrix4<f32> =
            cgmath::Matrix4::look_at(cgmath::Point3::new(5.0 * a.sin() + 0.5, 5.0 * a.cos() + 0.5, 2.0),
                                     cgmath::Point3::new(0.5, 0.5, 0.0),
                                     cgmath::vec3(0.0, 0.0, 1.0));

        // Convert to the format the shader expects.
        let projection: [[f32; 4]; 4] = projection.into();
        let modelview: [[f32; 4]; 4] = modelview.into();

        let uniforms = uniform! {
            projection: projection,
            modelview: modelview,
        };

        // Draw parameters

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        // Setup the cube model
        let faces = [Face::East,
                     Face::North,
                     Face::Up,
                     Face::West,
                     Face::South,
                     Face::Down];
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for f in 0..6 {
            let normal = faces[f].normal();
            let idx = vertices.len() as u16;
            for &v_idx in faces[f].vertices().into_iter() {
                vertices.push(Vertex {
                    pos: cube_vertex(v_idx),
                    normal: normal,
                    color: [1.0, 0.0, 0.0, 1.0],
                });

            }
            indices.push(idx);
            indices.push(idx + 1);
            indices.push(idx + 2);
            indices.push(idx);
            indices.push(idx + 2);
            indices.push(idx + 3);
        }
        let v_buf = glium::VertexBuffer::new(&window.display, &vertices).unwrap();
        let i_buf =
            glium::IndexBuffer::new(&window.display,
                                    glium::index::PrimitiveType::TrianglesList,
                                    &indices)
                .unwrap();


        // Draw the thing

        {
            let mut target = window.get_framebuffer_target();
            target.draw(&v_buf, &i_buf, &shader, &uniforms, &params).unwrap();
        }


        // window.display(&mut room);

        window.end_frame();

        tick += 1;
    }
}
