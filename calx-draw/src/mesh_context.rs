use glium::{Surface, Display};
use calx_cache::{AtlasBuilder, Atlas, AtlasItem};
use calx_window::{Displayable, Window};
use mesh::{Buffer, Vertex};

/// A rendering context that uses a dynamic mesh to build 2D graphics.
pub struct MeshContext {
    pub tiles: Vec<AtlasItem>,
    buffer: Buffer,
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
        }
    }

    pub fn add_mesh(&mut self,
                    vertices: Vec<Vertex>,
                    faces: Vec<[u16; 3]>) {
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
