extern crate rustc_serialize;
extern crate euclid;
#[macro_use]
extern crate glium;
extern crate toml;
extern crate vitral;
extern crate scancode;
#[macro_use]
extern crate calx_alg;
extern crate calx_color;
#[macro_use]
extern crate calx_resource;
extern crate calx_grid;
extern crate world;
extern crate display;

// Make all mods public at the top app level just to make them show up in the rustdoc.

pub mod mapedit_mod;

use euclid::{Point2D, Rect, Size2D};
use glium::{DisplayBuild, glutin};
use world::{Location, Terraform, World};
use mapedit_mod::View;

pub fn main() {
    // Construct display and Vitral context.
    let display = glutin::WindowBuilder::new()
                      .build_glium()
                      .unwrap();

    let mut backend = display::Backend::new(&display, 640, 360);

    // Initialize game resources.
    ::display::init::brushes(&display, &mut backend);
    ::display::init::font(&display, &mut backend);
    ::world::init::terrain();

    let mut context = display::Context {
        ui: vitral::Builder::new().build(|img| backend.make_texture(&display, img)),
        backend: backend,
    };

    // Initialize worldstate
    let mut view = View::new(World::new(1));

    for x in -0..8 {
        for y in 0..8 {
            view.world.set_terrain(Location::new(x, y, 0), 3);
        }
    }

    view.world.set_terrain(Location::new(0, -10, 0), 1);
    view.world.set_terrain(Location::new(1, 0, 0), 3);
    view.world.set_terrain(Location::new(2, 0, 0), 5);
    view.world.set_terrain(Location::new(3, 0, 0), 6);
    view.world.set_terrain(Location::new(4, 0, 0), 6);
    view.world.set_terrain(Location::new(3, 2, 0), 7);
    view.world.set_terrain(Location::new(4, 2, 0), 7);

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
