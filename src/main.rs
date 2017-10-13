// Don't show a console window when running on Windows.
#![windows_subsystem = "windows"]

extern crate rand;
extern crate euclid;
extern crate glium;
extern crate scancode;
extern crate vitral;
extern crate calx_grid;
#[macro_use]
extern crate calx_alg;
extern crate world;
extern crate display;

pub mod game_loop;

use euclid::{Point2D, Rect, Size2D};
use game_loop::GameLoop;
use glium::glutin;
use rand::Rng;
use vitral::Context;
use world::World;

pub fn main() {
    // Construct display and Vitral context.
    // XXX: Glium stuff needs to go into backend module...
    let events = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new().with_title("Magog");
    let context = glutin::ContextBuilder::new().with_gl(
        glutin::GlRequest::Specific(
            glutin::Api::OpenGl,
            (3, 2),
        ),
    );
    let display = glium::Display::new(window, context, &events).unwrap();

    let screen_area = Rect::new(Point2D::new(0.0, 0.0), Size2D::new(640.0f32, 360.0f32));
    let mut backend = display::Backend::new(
        &display,
        events,
        screen_area.size.width as u32,
        screen_area.size.height as u32,
    );

    let seed = rand::thread_rng().gen();
    // Print out the seed in case worldgen has a bug and we want to debug stuff with the same seed.
    println!("Seed: {}", seed);

    let mut game = GameLoop::new(World::new(seed));

    loop {
        backend.begin_frame();
        game.draw(&mut backend, &screen_area);
        if !backend.update(&display) {
            return;
        }
    }
}
