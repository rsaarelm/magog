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

mod backend;
mod game_view;
mod init;
mod render;
mod sprite;
mod view;

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

    let mut context: backend::Context;
    let mut builder = vitral::Builder::new();

    // Initialize game resources.
    init::brushes(&mut builder);
    init::terrain();

    context = builder.build(|img| backend.make_texture(&display, img));

    let font = context.default_font();

    // Initialize worldstate
    let mut view = GameView::new(World::new(1));

    view.world.terrain.set(Location::new(0, 0), 2);
    view.world.terrain.set(Location::new(0, -1), 1);
    view.world.terrain.set(Location::new(1, 0), 3);
    view.world.terrain.set(Location::new(2, 0), 5);
    view.world.terrain.set(Location::new(3, 0), 6);
    view.world.terrain.set(Location::new(4, 0), 6);
    view.world.terrain.set(Location::new(3, 2), 7);
    view.world.terrain.set(Location::new(4, 2), 7);

    // Run game.
    loop {
        context.begin_frame();

        let area = Rect::new(Point2D::new(0.0, 0.0), Size2D::new(640.0, 360.0));
        view.draw(&mut context, &area);

        context.draw_text(font,
                          Point2D::new(4.0, 20.0),
                          [1.0, 1.0, 1.0, 1.0],
                          "Hello, world!");

        if !backend.update(&display, &mut context) {
            return;
        }
    }
}
