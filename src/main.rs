extern crate euclid;
#[macro_use]
extern crate glium;
extern crate image;
extern crate vitral;
extern crate serde;
extern crate calx_color;
#[macro_use]
extern crate calx_resource;
extern crate calx_grid;
extern crate world;

pub mod backend;
pub mod game_view;
pub mod init;
pub mod render;
pub mod sprite;
pub mod view;

use euclid::{Point2D, Rect, Size2D};
use glium::{DisplayBuild, glutin};
use backend::Backend;
use world::World;
use world::Location;
use game_view::GameView;

pub fn main() {
    // Construct display and Vitral context.
    let display = glutin::WindowBuilder::new()
                      .build_glium()
                      .unwrap();

    let mut backend = Backend::new(&display);

    let mut builder = vitral::Builder::new();

    // Initialize game resources.
    init::brushes(&mut builder);
    init::terrain();

    let mut context = backend::Context {
        ui: builder.build(|img| backend.make_texture(&display, img)),
        backend: backend,
    };

    // Initialize worldstate
    let mut view = GameView::new(World::new(1));

    for x in -10..10 {
        for y in -10..10 {
            view.world.terrain.set(Location::new(x, y), 2);
        }
    }

    view.world.terrain.set(Location::new(0, -10), 1);
    view.world.terrain.set(Location::new(1, 0), 3);
    view.world.terrain.set(Location::new(2, 0), 5);
    view.world.terrain.set(Location::new(3, 0), 6);
    view.world.terrain.set(Location::new(4, 0), 6);
    view.world.terrain.set(Location::new(3, 2), 7);
    view.world.terrain.set(Location::new(4, 2), 7);

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
