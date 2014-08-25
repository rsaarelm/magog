use calx::color::consts::*;
use cgmath::{Vector2};
use calx::engine::{App, Engine, Key};
use view::tilecache;
use view::main::State;
use view::gamestate::GameState;

/// Title screen.
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
        let logo = tilecache::get(tilecache::LOGO);
        ctx.set_color(&FIREBRICK);
        ctx.draw_image(&logo, &Vector2::new(280f32, 180f32));
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
