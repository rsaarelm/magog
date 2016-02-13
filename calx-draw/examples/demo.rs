extern crate image;
extern crate glium;
extern crate calx_window;
extern crate calx_cache;
extern crate calx_draw;

use std::char;
use glium::Surface;
use calx_cache::{AtlasBuilder, Atlas, AtlasItem, ImageStore, Font};
use calx_window::{WindowBuilder, Window, Event, Key};
use calx_draw::Buffer;

struct Context {
    pub window: Window,
    pub buffer: Buffer,
    pub tiles: Vec<AtlasItem>,
    pub font: Font<usize>,
}

impl Context {
    pub fn new() -> Context {
        let window = WindowBuilder::new().set_title("Calx demo").build();
        let mut atlas_builder = AtlasBuilder::new();

        // Solid color as the default element.
        assert!(atlas_builder.add_solid_image() == Default::default());

        // Load font into atlas.
        const DATA: &'static [u8] = include_bytes!("../assets/font.png");
        let font = Font::new(&mut image::load_from_memory(DATA).unwrap(),
                             &(32..128)
                                  .map(|i| char::from_u32(i).unwrap())
                                  .collect::<String>(),
                             &mut atlas_builder);

        let Atlas {
            image: img,
            items: tiles,
        } = atlas_builder.build();

        let buffer = Buffer::new(&window.display, img);

        Context {
            window: window,
            buffer: buffer,
            tiles: tiles,
            font: font,
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
}

fn main() {
    let mut ctx = Context::new();

    'top: loop {
        for e in ctx.window.events().into_iter() {
            match e {
                Event::Quit => break 'top,
                Event::KeyPress(Key::Escape) => break 'top,
                _ => (),
            }
        }

        ctx.end_frame();
    }
}
