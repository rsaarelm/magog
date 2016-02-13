use glium::Surface;
use calx_cache::{AtlasBuilder, Atlas, AtlasItem};
use calx_window::{WindowBuilder, Window};
use mesh::{Buffer, Vertex};

/// A rendering context that uses a dynamic mesh to build 2D graphics.
pub struct MeshContext {
    pub window: Window,
    pub buffer: Buffer,
    pub tiles: Vec<AtlasItem>,
}

impl MeshContext {
    pub fn new(w: WindowBuilder, a: AtlasBuilder) -> MeshContext {
        let window = w.build();

        let Atlas {
            image: img,
            items: tiles,
        } = a.build();

        let buffer = Buffer::new(&window.display, img);

        MeshContext {
            window: window,
            buffer: buffer,
            tiles: tiles,
        }
    }

    pub fn end_frame(&mut self) {
        let display = self.window.display.clone();
        let buffer = &mut self.buffer;

        self.window.draw(|target| {
            target.clear_color(0.4, 0.6, 0.9, 0.0);
            target.clear_depth(1.0);
            buffer.flush(&display, target);
        });

        self.window.end_frame();
    }

    #[inline(always)]
    fn canvas_to_device(&self, pos: [f32; 3]) -> [f32; 3] {
        let size = self.window.size();
        [-1.0 + (2.0 * (pos[0]) / size[0] as f32),
          1.0 - (2.0 * (pos[1]) / size[1] as f32),
         pos[2]]
    }

    pub fn add_mesh(&mut self, mut vertices: Vec<Vertex>, faces: Vec<[u16; 3]>) {
        // Input is vertices in canvas pixel space, translate this into the
        // [-1.0, 1.0] device coordinate space.
        for v in vertices.iter_mut() {
            v.pos = self.canvas_to_device(v.pos);
        }
        self.buffer.add_mesh(vertices, faces);
    }
}
