use calx::color;
use calx::event;
use calx::key;
use world;
use super::{State, Transition};
use worldview;
use titlestate::TitleState;

pub struct GameState;

impl GameState {
    pub fn new(seed: Option<u32>) -> GameState {
        world::init_world(seed);
        GameState
    }
}

impl State for GameState {
    fn process(&mut self, event: event::Event) -> Option<Transition> {
        match event {
            event::Render(ctx) => {
                ctx.clear(&color::BLACK);
                let camera = world::camera();
                worldview::draw_world(&camera, ctx);
            }
            event::KeyPressed(key::KeyEscape) => {
                return Some(super::NewState(box TitleState::new()));
            }
            _ => ()
        }
        None
    }
}
