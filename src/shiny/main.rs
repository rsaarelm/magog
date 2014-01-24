extern mod calx;
extern mod sdl2;

pub fn main() {
    calx::hello();
    sdl2::init([sdl2::InitVideo]);
    println!("Shiny: A prototype user interface.");
    // TODO: Link to SDL or sth.
}

