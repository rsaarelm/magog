use euclid::{Point2D, Rect};
use world::{Location, World};
use display;

pub struct View {
    pub world: World,
}

impl View {
    pub fn new(world: World) -> View { View { world: world } }

    pub fn draw(&mut self, context: &mut display::Context, screen_area: &Rect<f32>) {
        let camera_loc = Location::new(0, 0, 0);
        let mut view = display::WorldView::new(camera_loc, *screen_area);
        view.show_cursor = true;

        view.draw(&self.world, context);

        let font = context.ui.default_font();
        if let Some(loc) = view.cursor_loc {
            let color =
            if world::on_screen(Point2D::new(loc.x as i32, loc.y as i32)) {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [1.0, 0.5, 0.5, 1.0]
            };
            context.ui.draw_text(&*font,
                                 Point2D::new(0.0, 16.0),
                                 color,
                                 &format!("Mouse pos {:?}", loc));
        }
    }
}
