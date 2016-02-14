use glium::{Surface, Display};
use calx_cache::{AtlasBuilder, Atlas, AtlasItem};
use calx_window::{Displayable, Window};
use mesh::{Buffer, Vertex};

/// A rendering context that uses a dynamic mesh to build 2D graphics.
pub struct MeshContext {
    pub tiles: Vec<AtlasItem>,
    buffer: Buffer,
    size: [u32; 2],
}

impl MeshContext {
    pub fn new(a: AtlasBuilder, window: &Window) -> MeshContext {
        let Atlas {
            image: img,
            items: tiles,
        } = a.build();

        let buffer = Buffer::new(&window.display, img);

        MeshContext {
            tiles: tiles,
            buffer: buffer,
            size: window.size(),
        }
    }

    #[inline(always)]
    fn canvas_to_device(&self, pos: [f32; 3]) -> [f32; 3] {
        [-1.0 + (2.0 * (pos[0]) / self.size[0] as f32),
         1.0 - (2.0 * (pos[1]) / self.size[1] as f32),
         pos[2]]
    }

    pub fn add_mesh(&mut self,
                    mut vertices: Vec<Vertex>,
                    faces: Vec<[u16; 3]>) {
        // Input is vertices in canvas pixel space, translate this into the
        // [-1.0, 1.0] device coordinate space.
        for v in vertices.iter_mut() {
            v.pos = self.canvas_to_device(v.pos);
        }
        self.buffer.add_mesh(vertices, faces);
    }
}

impl Displayable for MeshContext {
    fn display<S: Surface>(&mut self, display: &Display, target: &mut S) {
        target.clear_color(0.4, 0.6, 0.9, 0.0);
        target.clear_depth(1.0);
        self.buffer.flush(display, target);
    }
}
