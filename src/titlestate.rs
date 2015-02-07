use util::{V2, color};
use backend::{Key, Event};
use backend::{CanvasUtil};
use tilecache;

pub struct TitleState;

impl TitleState {
    pub fn new() -> TitleState { TitleState }

    pub fn process(&mut self, event: Event) -> bool {
        match event {
            Event::Render(ctx) => {
                ctx.draw_image(tilecache::get(tilecache::LOGO), V2(280.0, 180.0), 0.0, &color::FIREBRICK, &color::BLACK);
            }
            Event::KeyPressed(Key::Escape) => {
                return false;
            }
            Event::KeyPressed(_) => {
                return false;
            }
            _ => ()
        }
        true
    }
}
