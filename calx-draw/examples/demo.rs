extern crate image;
extern crate calx_window;
extern crate calx_cache;
extern crate calx_draw;

use calx_cache::{AtlasBuilder, Atlas, AtlasItem, tilesheet_bounds, subimage};
use calx_window::{WindowBuilder, Window, Event, Key};
use calx_draw::Buffer;

struct Context {
    pub window: Window,
    pub buffer: Buffer,
    pub tiles: Vec<AtlasItem>,
}

impl Context {
    pub fn new() -> Context {
        let window = WindowBuilder::new().set_title("Calx demo").build();
        let mut atlas_builder = AtlasBuilder::new();

        // Solid color as the default element.
        assert!(atlas_builder.push_solid() == Default::default());

        let mut sheet = image::load_from_memory(include_bytes!("../assets/font.png")).unwrap();
        let bounds = tilesheet_bounds(&sheet);


        let Atlas {
            image: img,
            items: tiles,
        } = atlas_builder.build();

        let buffer = Buffer::new(&window.display, img);
        Context {
            window: window,
            buffer: buffer,
            tiles: tiles,
        }
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

        ctx.window.end_frame();
    }
}
