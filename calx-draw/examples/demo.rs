extern crate image;
extern crate glium;
extern crate calx_color;
extern crate calx_layout;
extern crate calx_window;
extern crate calx_cache;
extern crate calx_draw;

use std::char;
use calx_color::{color};
use calx_layout::Rect;
use calx_cache::{AtlasBuilder, ImageStore, Font};
use calx_window::{WindowBuilder, Event, Key};
use calx_draw::{MeshContext, DrawUtil};

fn main() {
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

    let mut ctx = MeshContext::new(WindowBuilder::new().set_title("Calx demo"),
                                   atlas_builder);

    'top: loop {
       ctx.draw_image(10, [10.0, 10.0], 0.4, color::BLACK, color::BLACK);

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
