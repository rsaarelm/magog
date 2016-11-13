extern crate rand;
extern crate euclid;
extern crate glium;
extern crate scancode;
extern crate vitral;
extern crate calx_grid;
extern crate calx_resource;
extern crate world;
extern crate display;

pub mod game_view;

use rand::{XorShiftRng, SeedableRng};
use euclid::{Point2D, Rect, Size2D};
use glium::{DisplayBuild, glutin};
use world::{World, Location, mapgen};
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
    mapgen::caves(&mut world, &mut XorShiftRng::from_seed([seed, 1, 1, 1]), Location::new(0, 0, 0), 300);

    let mut view = View::new(world);

    loop {
        context.ui.begin_frame();

        view.draw(&mut context, &screen_area);

        if !context.backend.update(&glium, &mut context.ui) {
            return;
        }
    }
}
