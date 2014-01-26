extern mod calx;
extern mod sdl2;

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

    app.pixels[1] = 0xff;
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

