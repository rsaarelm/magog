#![crate_name="magog"]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(tuple_indexing)]
#![comment = "Magog toplevel and display interface"]

extern crate image;
extern crate calx;
extern crate world;
extern crate time;

use calx::event;
use titlestate::TitleState;

pub mod drawable;
pub mod tilecache;
pub mod viewutil;
pub mod worldview;
mod gamestate;
mod titlestate;

pub trait State {
    fn process(&mut self, event: event::Event) -> Option<Transition>;
}

pub enum Transition {
    NewState(Box<State + Send>),
    Quit,
}

pub fn main() {
    let mut canvas = calx::Canvas::new();
    tilecache::init(&mut canvas);
    let mut state: Box<State + Send> = box TitleState::new();

    for evt in canvas.run() {
        match state.process(evt) {
            Some(Quit) => { return; }
            Some(NewState(s)) => { state = s; }
            _ => ()
        }
    }
}
