extern crate euclid;
extern crate glium;
extern crate vitral;
extern crate scancode;
#[macro_use]
extern crate calx;
extern crate world;
extern crate display;

// Make all mods public at the top app level just to make them show up in the rustdoc.

pub mod mapedit_mod;

use euclid::{Point2D, Rect, Size2D};
use glium::{DisplayBuild, glutin};
use mapedit_mod::View;
use vitral::Context;
use world::World;

pub fn main() {
    // Construct display and Vitral context.
    let display = glutin::WindowBuilder::new().build_glium().unwrap();

    let mut backend = display::Backend::new(&display, 640, 480);

    // Initialize worldstate
    let mut view = View::new(World::new(1));

    // Run game.
    loop {
        backend.begin_frame();

        let area = Rect::new(Point2D::new(0.0, 0.0), Size2D::new(640.0, 360.0));
        view.draw(&mut backend, &area);

        if !backend.update(&display) {
            return;
        }
    }
}
