extern crate time;
extern crate glium;
extern crate image;
extern crate calx;

use glium::{Surface};
use calx::{scolor};
use calx::backend::{WindowBuilder, mesh, RenderTarget};

fn main() {
    let mut window =
        WindowBuilder::new()
        .set_title("Window demo")
        .set_frame_interval(0.1)
        .build();

    let pixel: image::Rgba<u8> = scolor::YELLOW.into();
    let atlas = image::ImageBuffer::from_pixel(32, 32, pixel);
    let mut buffer = mesh::Buffer::new(&window.display, atlas);
    loop {
        buffer.add_mesh(
            vec![
                mesh::Vertex{ pos: [0.0, 0.0, 0.5], tex_coord: [0.0, 0.0], color: [1.0, 1.0, 1.0, 1.0], back_color: [0.0, 0.0, 0.0, 0.0] },
                mesh::Vertex{ pos: [0.0, 1.0, 0.5], tex_coord: [0.0, 0.0], color: [1.0, 1.0, 1.0, 1.0], back_color: [0.0, 0.0, 0.0, 0.0] },
                mesh::Vertex{ pos: [1.0, 0.0, 0.5], tex_coord: [0.0, 0.0], color: [1.0, 1.0, 1.0, 1.0], back_color: [0.0, 0.0, 0.0, 0.0] },
            ],

            vec![[0, 1, 2]]);

        window.draw(|target| {
            target.clear_color(1.0, 0.0, 0.0, 1.0);
            target.clear_depth(1.0);
            buffer.flush(&window.display, target);
        });

        for event in window.events().into_iter() {
            use calx::backend::Event::*;
            use calx::backend::Key;
            match event {
                Quit => { return; }
                KeyPress(Key::Escape) => { return; }
                _ => {}
            }
        }

        window.end_frame();
    }
}
