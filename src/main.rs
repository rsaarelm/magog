#![crate_name="magog"]
#![allow(unstable)]

extern crate image;
extern crate "calx_util" as util;
extern crate "calx_backend" as backend;
extern crate world;
extern crate time;

use gamestate::GameState;

pub static SCREEN_W: i32 = 640;
pub static SCREEN_H: i32 = 360;

pub mod drawable;
pub mod tilecache;
pub mod viewutil;
pub mod worldview;
mod gamestate;
//mod titlestate;
mod sprite;
mod msg_queue;

// TODO Fix state machine code.
/*
pub trait State {
    fn process(&mut self, event: event::Event) -> Option<Transition>;
}

pub enum Transition {
    NewState(State),
    Quit,
}
*/

pub fn main() {
    let mut canvas = backend::CanvasBuilder::new()
        .set_dim(util::V2(SCREEN_W, SCREEN_H))
        .set_frame_interval(0.030f64);
    tilecache::init(&mut canvas);
    let mut state = GameState::new(None);

    for evt in canvas.run() {
        match state.process(evt) {
            false => { return; }
            _ => ()
        }
    }
}
