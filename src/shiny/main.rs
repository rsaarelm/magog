extern mod stb;

extern mod glfw;

//use std::io::File;

pub fn main() {
    println!("Shiny: A prototype user interface.");
    do glfw::start {
        let window = glfw::Window::create(800, 600, "Shiny!", glfw::Windowed)
            .expect("Failed to create window.");
        window.make_context_current();
        while !window.should_close() {
            glfw::poll_events();
        }
    }
    /*
    let font = stb::truetype::Font::new(
        File::open(&Path::new("assets/pf_tempesta_seven_extended_bold.ttf")).read_to_end())
        .unwrap();

    let glyph = font.glyph(80, 13.0).unwrap();

    for y in range(0, glyph.height) {
        for x in range(0, glyph.width) {
            app.pixels[x * 4 + y * 4 * 800 + 1] = glyph.pixels[x + y * glyph.width];
        }
    }
    */
}
