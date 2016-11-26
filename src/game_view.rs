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
            Num1 => {
                writeln!(&mut self.console, "PRINTAN!").unwrap();
                Ok(())
            }
            Num2 => {
                writeln!(&mut self.console,
                         "The quick brown fox jumps over the lazy dog.")
                    .unwrap();
                Ok(())
            }
            Num3 => {
                writeln!(&mut self.console,
                         "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Cras \
                         volutpat, diam eget iaculis ullamcorper, diam ligula cursus velit, \
                         et tristique nisl mi eu lectus. Vestibulum ullamcorper lectus sed \
                         magna tempor condimentum. Sed lorem metus, ultrices vitae nunc in, \
                         luctus ultricies lorem. Proin at risus et arcu interdum tempor \
                         quis eu urna. Phasellus lectus neque, finibus vitae enim eget, \
                         tincidunt venenatis nisl. In aliquet at leo et mollis. Mauris \
                         sollicitudin leo metus, nec finibus quam ultrices at. Quisque \
                         lacinia hendrerit placerat. Sed nec enim sit amet nulla volutpat \
                         vestibulum quis lacinia lectus. Morbi at diam at sapien efficitur \
                         rutrum. Quisque pellentesque nulla non erat viverra, eget tempus \
                         lorem pharetra. Aenean lobortis ut elit et varius. Phasellus nisl \
                         orci, mollis non tincidunt quis, lacinia non diam. Morbi in \
                         scelerisque eros. Quisque quis augue et enim pretium ullamcorper \
                         et ac arcu.")
                    .unwrap();
                Ok(())
            }
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
                if let Err(e) = self.parse(&input) { writeln!(&mut self.console, "{}", e); }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn ping(&mut self) {
        writeln!(&mut self.console, "pong");
    }

    command_parser!{
        fn ping(&mut self);
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
