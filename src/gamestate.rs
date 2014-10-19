use calx::color;
use calx::Context;
use calx::event;
use calx::key;
use world;
use world::action;
use world::action::{Step};
use world::dir6::*;
use super::{State, Transition};
use worldview;
use titlestate::TitleState;

pub struct GameState;

impl GameState {
    pub fn new(seed: Option<u32>) -> GameState {
        world::init_world(seed);
        GameState
    }

    /// Repaint view, update game world if needed.
    pub fn update(&mut self, ctx: &mut Context) {
        ctx.clear(&color::BLACK);
        let camera = world::camera();
        worldview::draw_world(&camera, ctx);

        if action::control_state() == action::ReadyToUpdate {
            action::update();
        }
    }

    /// Process a player control keypress.
    pub fn process_key(&mut self, key: key::Key) -> bool {
        if action::control_state() != action::AwaitingInput {
            return false;
        }

        match key {
            key::KeyQ | key::KeyPad7 => { action::input(Step(NorthWest)); }
            key::KeyW | key::KeyPad8 | key::KeyUp => { action::input(Step(North)); }
            key::KeyE | key::KeyPad9 => { action::input(Step(NorthEast)); }
            key::KeyA | key::KeyPad1 => { action::input(Step(SouthWest)); }
            key::KeyS | key::KeyPad2 | key::KeyDown => { action::input(Step(South)); }
            key::KeyD | key::KeyPad3 => { action::input(Step(SouthEast)); }
            _ => { return false; }
        }
        return true;
    }
}

impl State for GameState {
    fn process(&mut self, event: event::Event) -> Option<Transition> {
        match event {
            event::Render(ctx) => {
                self.update(ctx);
            }
            event::KeyPressed(key::KeyEscape) => {
                return Some(super::NewState(box TitleState::new()));
            }
            event::KeyPressed(k) => {
                self.process_key(k);
            }
            _ => ()
        }
        None
    }
}
