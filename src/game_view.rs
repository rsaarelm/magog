use euclid::{Point2D, Rect};
use calx_grid::Dir6;
use scancode::Scancode;
use world::{Location, World, TerrainQuery, Command};
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
            if self.world.is_valid_location(loc) {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [1.0, 0.5, 0.5, 1.0]
            };
            context.ui.draw_text(&*font,
                                 Point2D::new(0.0, 16.0),
                                 color,
                                 &format!("Mouse pos {:?}", loc));
        }

        if let Some(scancode) = context.backend.poll_key().and_then(|k| Scancode::new(k.scancode)) {
            use scancode::Scancode::*;
            match scancode {
                Q => self.world.step(Dir6::Northwest),
                W => self.world.step(Dir6::North),
                E => self.world.step(Dir6::Northeast),
                A => self.world.step(Dir6::Southwest),
                S => self.world.step(Dir6::South),
                D => self.world.step(Dir6::Southeast),
                _ => Ok(())
            };
        }
    }
}
