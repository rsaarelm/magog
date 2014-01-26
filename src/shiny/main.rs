extern mod calx;
extern mod stb;
extern mod sdl2;

use std::io::File;

use sdl2::event;
use sdl2::keycode;
use calx::app::App;

pub fn main() {
    calx::hello();
    println!("Shiny: A prototype user interface.");
    let mut app = match App::new(~"Shiny!", 800, 600) {
        Ok(app) => app,
        Err(err) => fail!(err)
    };

    let font = stb::truetype::Font::new(
        File::open(&Path::new("assets/pf_tempesta_seven_extended_bold.ttf")).read_to_end())
        .unwrap();

    let glyph = font.glyph(80, 13.0).unwrap();

    for y in range(0, glyph.height) {
        for x in range(0, glyph.width) {
            app.pixels[x * 4 + y * 4 * 800 + 1] = glyph.pixels[x + y * glyph.width];
        }
    }

    app.render();

    loop {
        match event::poll_event() {
            event::QuitEvent(_) => break,
            event::KeyDownEvent(_, _, key, _, _) => {
                if key == keycode::EscapeKey {
                    break
                }
            },
            event::WindowEvent(_, _, _, _, _) => {
                app.render();
            }
            _ => {}
        }
    }
}
