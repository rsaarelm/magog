#![crate_name="magog"]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(tuple_indexing)]
#![feature(if_let)]

extern crate image;
extern crate calx;
extern crate world;
extern crate time;

use gamestate::GameState;

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
    let mut canvas = calx::Canvas::new()
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
