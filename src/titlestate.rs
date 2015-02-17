use util::{V2, color};
use backend::{Key, Event};
use backend::{CanvasUtil};
use tilecache;
use ::{Transition, State};

pub struct TitleState;

impl TitleState {
    pub fn new() -> TitleState { TitleState }

}

impl State for TitleState {
    fn process(&mut self, event: Event) -> Option<Transition> {
        match event {
            Event::Render(ctx) => {
                ctx.draw_image(tilecache::get(tilecache::LOGO), V2(280.0, 180.0), 0.0, &color::FIREBRICK, &color::BLACK);
            }
            Event::KeyPressed(Key::Escape) => {
                return Some(Transition::Exit);
            }
            Event::KeyPressed(_) => {
                return Some(Transition::Game(None));
            }
            _ => ()
        }
        None
    }
}
