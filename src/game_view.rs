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

        view.draw(&self.world, context);

        let font = context.ui.default_font();
        if let Some(loc) = view.cursor_loc {
            context.ui.draw_text(&*font,
                                 Point2D::new(0.0, 16.0),
                                 [1.0, 1.0, 1.0, 1.0],
                                 &format!("Mouse pos {:?}", loc));
        }
    }
}
