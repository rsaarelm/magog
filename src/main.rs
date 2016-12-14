extern crate rand;
extern crate euclid;
extern crate glium;
extern crate scancode;
extern crate vitral;
extern crate calx_grid;
extern crate calx_resource;
#[macro_use]
extern crate calx_alg;
extern crate world;
extern crate display;

pub mod game_view;

use std::fs::File;
use rand::{SeedableRng, XorShiftRng};
use euclid::{Point2D, Rect, Size2D};
use glium::{DisplayBuild, glutin};
use world::{Location, World, mapgen, Mutate};
use game_view::View;

pub fn main() {
    // Construct display and Vitral context.
    let glium = glutin::WindowBuilder::new()
                    .build_glium()
                    .unwrap();

    let screen_area = Rect::new(Point2D::new(0.0, 0.0), Size2D::new(640.0f32, 360.0f32));
    let mut backend = display::Backend::new(&glium,
                                            screen_area.size.width as u32,
                                            screen_area.size.height as u32);

    // Initialize game resources.
    ::display::init::brushes(&glium, &mut backend);
    ::display::init::font(&glium, &mut backend);
    ::world::init::terrain();

    let mut context = display::Context {
        ui: vitral::Builder::new().build(|img| backend.make_texture(&glium, img)),
        backend: backend,
    };

    let seed = 1;

    let mut world = World::new(seed);

    /// TODO error handling.
    let prefab = world::load_prefab(&mut File::open("sprint.toml").unwrap()).unwrap();
    world.deploy_prefab(Location::new(-21, -22, 0), &prefab);

    let mut view = View::new(world);

    loop {
        context.ui.begin_frame();

        view.draw(&mut context, &screen_area);

        if !context.backend.update(&glium, &mut context.ui) {
            return;
        }
    }
}
