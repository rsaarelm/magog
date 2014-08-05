use cgmath::point::{Point2};
use calx::engine::{App, Engine, Key};
use game::main::State;
use game::gamestate::GameState;

pub struct TitleState {
    running: bool,
}

impl TitleState {
    pub fn new() -> TitleState {
        TitleState {
            running: true,
        }
    }
}

impl App for TitleState {
    fn setup(&mut self, _ctx: &mut Engine) {
    }

    fn draw(&mut self, ctx: &mut Engine) {
        ctx.draw_string("MAGOG", &Point2::new(0f32, 8f32));
    }

    fn char_typed(&mut self, _ctx: &mut Engine, _ch: char) {
        self.running = false;
    }

    fn key_pressed(&mut self, _ctx: &mut Engine, _key: Key) {}
    fn key_released(&mut self, _ctx: &mut Engine, _key: Key) {}
}

impl State for TitleState {
    fn next_state(&self) -> Option<Box<State>> {
        if !self.running {
            Some(box GameState::new() as Box<State>)
        } else {
            None
        }
    }
}
