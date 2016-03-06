#[macro_use]
extern crate glium;
extern crate rand;
extern crate noise;
extern crate calx_system;
extern crate calx_window;
extern crate cgmath;

use std::default::Default;
use calx_window::{WindowBuilder, Event, Key};
use cgmath::Angle;
use glium::Surface;
use rand::random;

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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Voxel {
    Dirt,
}

impl Voxel {
    pub fn color(&self, face: Face) -> [f32; 4] {
        match self {
            &Voxel::Dirt => {
                match face {
                    Face::Up => [0.0, 0.6, 0.0, 1.0],
                    _ => [0.5, 0.3, 0.0, 1.0],
                }
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct VoxelPos {
    x: i16,
    y: i16,
    z: i8,
}

impl VoxelPos {
    pub fn new(x: i16, y: i16, z: i8) -> VoxelPos {
        VoxelPos { x: x, y: y, z: z }
    }
}

struct World {
    seed: noise::Seed,
}


impl World {
    pub fn new() -> World {
        World { seed: random() }
    }

    fn ground_height(&self, x: f32, y: f32) -> f32 {
        4.0 * noise::perlin2(&self.seed, &[x / 14.1, y / 14.1])
    }

    pub fn get_voxel(&self, voxel_pos: &VoxelPos) -> Option<Voxel> {
        if (voxel_pos.z as f32) <
           self.ground_height(voxel_pos.x as f32, voxel_pos.y as f32) {
            Some(Voxel::Dirt)
        } else {
            None
        }
    }
}

struct Chunk {
    pub vtx: glium::VertexBuffer<Vertex>,
    pub idx: glium::IndexBuffer<u16>,
}

impl Chunk {
    pub fn new(display: &glium::Display,
               world: &World,
               min: VoxelPos,
               max: VoxelPos)
               -> Chunk {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let faces = [Face::East,
                     Face::North,
                     Face::Up,
                     Face::West,
                     Face::South,
                     Face::Down];

        for z in min.z..max.z {
            for y in min.y..max.y {
                for x in min.x..max.x {
                    let pos = VoxelPos::new(x, y, z);
                    if let Some(voxel) = world.get_voxel(&pos) {

                        for f in 0..6 {
                            let normal = faces[f].normal();
                            let neighbor = VoxelPos::new(pos.x +
                                                         normal[0] as i16,
                                                         pos.y +
                                                         normal[1] as i16,
                                                         pos.z +
                                                         normal[2] as i8);
                            if world.get_voxel(&neighbor).is_some() {
                                continue;
                            }

                            let idx = vertices.len() as u16;
                            for &v_idx in faces[f].vertices().into_iter() {
                                let p = cube_vertex(v_idx);
                                vertices.push(Vertex {
                                    pos: [p[0] + pos.x as f32,
                                          p[1] + pos.y as f32,
                                          p[2] + pos.z as f32],
                                    normal: normal,
                                    color: voxel.color(faces[f]),
                                });

                            }
                            indices.push(idx);
                            indices.push(idx + 1);
                            indices.push(idx + 2);
                            indices.push(idx);
                            indices.push(idx + 2);
                            indices.push(idx + 3);
                        }
                    }
                }
            }
        }

        Chunk {
            vtx: glium::VertexBuffer::new(display, &vertices).unwrap(),
            idx: glium::IndexBuffer::new(display,
                                    glium::index::PrimitiveType::TrianglesList,
                                    &indices)
                .unwrap()
        }
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
                const vec3 LIGHT = vec3(-0.2, 0.1, 0.8);

                void main() {
                    float lum = max(dot(normalize(v_normal), normalize(LIGHT)), 0.0);
                    f_color = (0.3 + 0.7 * lum) * v_color;
                }
            ",
            }
        ).unwrap();

    let mut tick = 0;

    let world = World::new();
    let chunk = Chunk::new(&window.display,
                           &world,
                           VoxelPos::new(0, 0, -16),
                           VoxelPos::new(32, 32, 16));

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

        let a = (tick as f32) / 96.0;
        let modelview: cgmath::Matrix4<f32> =
            cgmath::Matrix4::look_at(cgmath::Point3::new(15.0 * a.sin() +
                                                         16.0,
                                                         15.0 * a.cos() +
                                                         16.0,
                                                         16.0),
                                     cgmath::Point3::new(16.0, 16.0, 0.0),
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
            backface_culling: glium::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        // Draw the thing

        {
            let mut target = window.get_framebuffer_target();
            target.draw(&chunk.vtx, &chunk.idx, &shader, &uniforms, &params)
                  .unwrap();
        }


        // window.display(&mut room);

        window.end_frame();

        tick += 1;
    }
}
