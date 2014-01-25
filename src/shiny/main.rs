extern mod calx;
extern mod sdl2;

pub fn main() {
    calx::hello();
    sdl2::init([sdl2::InitVideo]);
    println!("Shiny: A prototype user interface.");
    let window = match sdl2::video::Window::new("Shiny!",
        sdl2::video::PosCentered, sdl2::video::PosCentered,
        800, 600, [sdl2::video::OpenGL]) {
        Ok(window) => window,
        Err(err) => fail!(format!("SDL2 window fail: {}", err))
    };

    let renderer = match sdl2::render::Renderer::from_window(window, sdl2::render::DriverAuto, [sdl2::render::Accelerated]) {
        Ok(renderer) => renderer,
        Err(err) => fail!(format!("SDL2 renderer fail: {}", err))
    };

    renderer.set_draw_color(sdl2::pixels::RGB(16, 0, 0));
    renderer.clear();
    renderer.present();

    loop {
        match sdl2::event::poll_event() {
            sdl2::event::QuitEvent(_) => break,
            sdl2::event::KeyDownEvent(_, _, key, _, _) => {
                if key == sdl2::keycode::EscapeKey {
                    break
                }
            },
            sdl2::event::WindowEvent(_, _, _, _, _) => {
                renderer.clear();
                renderer.present();
            }
            _ => {}
        }
    }
}

