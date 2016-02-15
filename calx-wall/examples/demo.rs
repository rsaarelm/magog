extern crate image;
extern crate glium;
extern crate calx_system;
extern crate calx_color;
extern crate calx_window;
extern crate calx_cache;
extern crate calx_wall;

use std::char;
use calx_color::color;
use calx_cache::{AtlasBuilder, ImageStore};
use calx_window::{WindowBuilder, Event, Key};
use calx_wall::{Wall, Font, Fonter};

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

    let mut window = WindowBuilder::new().set_title("Calx demo").build();
    let mut wall = Wall::new(&window.display, atlas_builder);

    'top: loop {
        for e in window.events().into_iter() {
            match e {
                Event::Quit => break 'top,
                Event::KeyPress(Key::Escape) => break 'top,
                Event::KeyPress(Key::F12) => {
                    calx_system::save_screenshot("calx", window.screenshot())
                        .unwrap();
                }
                _ => (),
            }
        }

        window.clear(0x7799DDFF);

        {
            let mut fonter = Fonter::new(&mut wall, &font)
                                 .width(128.0)
                                 .color(color::WHITE)
                                 .border(color::BLACK)
                                 .layer(0.4)
                                 .text("The quick brown fox jumps over the \
                                        lazy dog"
                                           .to_string());

            fonter.draw([48.0, 48.0]);
        }
        window.display(&mut wall);

        window.end_frame();

    }
}
