extern crate rustc_serialize;
extern crate euclid;
#[macro_use]
extern crate glium;
extern crate vitral;
extern crate scancode;
#[macro_use]
extern crate calx_alg;
extern crate calx_color;
#[macro_use]
extern crate calx_resource;
extern crate calx_ecs;
extern crate calx_grid;
extern crate world;
extern crate display;

// Make all mods public at the top app level just to make them show up in the rustdoc.

pub mod mapedit_mod;

use euclid::{Point2D, Rect, Size2D};
use glium::{DisplayBuild, glutin};
use world::World;
use mapedit_mod::View;

pub fn main() {
    // Construct display and Vitral context.
    let display = glutin::WindowBuilder::new()
                      .build_glium()
                      .unwrap();

    let mut backend = display::Backend::new(&display, 640, 480);

    // Initialize game resources.
    ::display::init::brushes(&display, &mut backend);
    ::display::init::font(&display, &mut backend);

    let mut context = display::Context {
        ui: vitral::Builder::new().build(|img| backend.make_texture(&display, img)),
        backend: backend,
    };

    // Initialize worldstate
    let mut view = View::new(World::new(1));

    // Run game.
    loop {
        context.ui.begin_frame();

        let area = Rect::new(Point2D::new(0.0, 0.0), Size2D::new(640.0, 360.0));
        view.draw(&mut context, &area);

        if !context.backend.update(&display, &mut context.ui) {
            return;
        }
    }
}
