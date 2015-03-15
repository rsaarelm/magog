use util::{V2, color, Anchor};
use backend::{Key, Event};
use backend::{CanvasUtil, Fonter, Align};
use tilecache;
use ::{Transition, State};

pub struct TitleState {
    screenshot_requested: bool,
}

impl TitleState {
    pub fn new() -> TitleState {
        TitleState {
            screenshot_requested: false,
        }
    }
}

impl State for TitleState {
    fn process(&mut self, event: Event) -> Option<Transition> {
        match event {
            Event::Render(ctx) => {
                if self.screenshot_requested {
                    ::screenshot(ctx);
                    self.screenshot_requested = false;
                }

                ctx.draw_image(tilecache::get(tilecache::LOGO), V2(274.0, 180.0), 0.0, &color::FIREBRICK, &color::BLACK);
                Fonter::new(ctx)
                    .color(&color::DARKRED)
                    .anchor(Anchor::Bottom)
                    .align(Align::Center)
                    .text(format!("Copyright (C) Risto Saarelma 2011 - 2015\nv{}{}", ::version(), if !cfg!(ndebug) { " debug" } else { "" }))
                    .draw(V2(320.0, 352.0));
            }
            Event::KeyPressed(Key::Escape) => {
                return Some(Transition::Exit);
            }
            Event::KeyPressed(Key::F12) => { self.screenshot_requested = true; }
            Event::KeyPressed(_) => {
                return Some(Transition::Game(None));
            }
            _ => ()
        }
        None
    }
}
