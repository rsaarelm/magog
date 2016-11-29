use std::io::Write;
use euclid::{Point2D, Rect};
use rand;
use calx_grid::Dir6;
use calx_resource::Resource;
use scancode::Scancode;
use world::{on_screen, Command, Location, TerrainQuery, World};
use display;

pub struct View {
    pub world: World,
    pub console: display::Console,
    pub console_is_large: bool,
}

impl View {
    pub fn new(world: World) -> View {
        View {
            world: world,
            console: display::Console::new(),
            console_is_large: false,
        }
    }

    fn game_input(&mut self, scancode: Scancode) -> Result<(), ()> {
        use scancode::Scancode::*;
        match scancode {
            Tab => Ok(self.console_is_large = !self.console_is_large),
            Q => self.world.step(Dir6::Northwest),
            W => self.world.step(Dir6::North),
            E => self.world.step(Dir6::Northeast),
            A => self.world.step(Dir6::Southwest),
            S => self.world.step(Dir6::South),
            D => self.world.step(Dir6::Southeast),
            _ => Ok(()),
        }
    }

    fn console_input(&mut self, scancode: Scancode) -> Result<(), ()> {
        use scancode::Scancode::*;
        match scancode {
            Tab => Ok(self.console_is_large = !self.console_is_large),
            Enter | PadEnter => {
                let input = self.console.get_input();
                writeln!(&mut self.console, "{}", input);
                if let Err(e) = self.parse(&input) {
                    writeln!(&mut self.console, "{}", e);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn dump(&mut self) {
        dump_map(&self.world);
    }

    /// Generate a new random cave map.
    fn cave(&mut self) {
        use world::mapgen;
        self.world = World::new(1);
        mapgen::caves(&mut self.world,
                      &mut rand::thread_rng(),
                      Location::new(0, 0, 0),
                      300);
    }

    /// Generate a new random maze map.
    fn maze(&mut self, sparseness: usize) {
        use world::mapgen;
        self.world = World::new(1);
        mapgen::maze(&mut self.world, &mut rand::thread_rng(), sparseness);
    }

    /// Generate a new random rooms and corridors
    fn rooms(&mut self) {
        use world::mapgen;
        self.world = World::new(1);
        mapgen::rooms(&mut self.world, &mut rand::thread_rng());
    }

    command_parser!{
        fn cave(&mut self);
        fn maze(&mut self, sparseness: usize);
        fn rooms(&mut self);

        fn dump(&mut self);
    }

    fn parse_command(&mut self, command: &str) {
        writeln!(&mut self.console,
                 "TODO: Do something clever with '{}' here.",
                 command);
    }

    pub fn draw(&mut self, context: &mut display::Context, screen_area: &Rect<f32>) {
        let camera_loc = Location::new(0, 0, 0);
        let mut view = display::WorldView::new(camera_loc, *screen_area);
        view.show_cursor = true;

        view.draw(&self.world, context);

        if self.console_is_large {
            let mut console_area = *screen_area;
            console_area.size.height /= 2.0;
            self.console.draw_large(context, &console_area);
        } else {
            self.console.draw_small(context, screen_area);
        }

        if let Some(scancode) = context.backend.poll_key().and_then(|k| Scancode::new(k.scancode)) {
            if self.console_is_large {
                self.console_input(scancode)
            } else {
                self.game_input(scancode)
            };
        }
    }
}

/// Print the world map as ASCII.
fn dump_map(world: &World) {
    for y in -21..21 {
        for x in -39..41 {
            if (x + y) % 2 != 0 { print!(" "); continue; }
            let pos = Point2D::new((x + y) / 2, y);
            if on_screen(pos) {
                let t = world.terrain(Location::new(0, 0, 0) + pos);
                if t.is_open() {
                    print!(".");
                } else if t.is_door() {
                    print!("+");
                } else if t.is_wall() {
                    print!("#");
                } else {
                    print!("*");
                }
            } else {
                print!(" ");
            }
        }
        println!("");
    }
}
