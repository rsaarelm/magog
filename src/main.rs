extern crate euclid;
extern crate glium;
extern crate scancode;
extern crate vitral;
extern crate calx_grid;
extern crate calx_resource;
extern crate world;
extern crate display;
extern crate content;

pub mod game_view;

use euclid::{Point2D, Rect, Size2D};
use glium::{DisplayBuild, glutin};
use world::World;
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
    content::init_brushes(&glium, &mut backend);
    content::init_terrain();

    let mut context = display::Context {
        ui: vitral::Builder::new().build(|img| backend.make_texture(&glium, img)),
        backend: backend,
    };

    let mut view = View::new(World::new(1));

    loop {
        context.ui.begin_frame();

        view.draw(&mut context, &screen_area);

        if !context.backend.update(&glium, &mut context.ui) {
            return;
        }
    }
}
