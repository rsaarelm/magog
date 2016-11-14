use std::io::Write;
use euclid::{Point2D, Rect};
use calx_grid::Dir6;
use calx_resource::Resource;
use scancode::Scancode;
use world::{Command, Location, TerrainQuery, World};
use display;

pub struct View {
    pub world: World,
    pub console: display::Console,
}

impl View {
    pub fn new(world: World) -> View {
        View {
            world: world,
            console: display::Console::new(),
        }
    }

    pub fn draw(&mut self, context: &mut display::Context, screen_area: &Rect<f32>) {
        let camera_loc = Location::new(0, 0, 0);
        let mut view = display::WorldView::new(camera_loc, *screen_area);
        view.show_cursor = true;

        view.draw(&self.world, context);

        // TODO: State toggle to use big console.
        self.console.draw_small(context, screen_area);

        if let Some(scancode) = context.backend.poll_key().and_then(|k| Scancode::new(k.scancode)) {
            use scancode::Scancode::*;
            match scancode {
                Q => self.world.step(Dir6::Northwest),
                W => self.world.step(Dir6::North),
                E => self.world.step(Dir6::Northeast),
                A => self.world.step(Dir6::Southwest),
                S => self.world.step(Dir6::South),
                D => self.world.step(Dir6::Southeast),
                Num1 => { writeln!(&mut self.console, "PRINTAN!").unwrap(); Ok(()) }
                Num2 => { writeln!(&mut self.console, "The quick brown fox jumps over the lazy dog.").unwrap(); Ok(()) }
                _ => Ok(()),
            };
        }
    }
}
