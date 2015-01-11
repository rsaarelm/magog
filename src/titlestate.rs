use util::{V2, color};
use backend::{key, event};
use super::{State, Transition};
use gamestate::GameState;
use tilecache;

pub struct TitleState;

impl TitleState {
    pub fn new() -> TitleState { TitleState }
}

impl State for TitleState {
    fn process(&mut self, event: event::Event) -> Option<Transition> {
        match event {
            event::Render(ctx) => {
                ctx.clear(&color::BLACK);
                ctx.draw_image(V2(280, 180), 0.0, tilecache::get(tilecache::LOGO), &color::FIREBRICK);
            }
            event::KeyPressed(key::KeyEscape) => {
                return Some(super::Quit);
            }
            event::KeyPressed(_) => {
                return Some(super::NewState(box GameState::new(None)));
            }
            _ => ()
        }
        None
    }
}
