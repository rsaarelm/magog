use calx::{V2, color, Anchor};
use calx::backend::{Key, Event};
use calx::backend::{Canvas, CanvasUtil, Fonter, Align};
use tilecache;
use ::{Transition, State};

pub struct TitleState;

impl TitleState {
    pub fn new() -> TitleState { TitleState }
}

impl State for TitleState {
    fn process(&mut self, ctx: &mut Canvas, event: Event) -> Option<Transition> {
        match event {
            Event::RenderFrame => {
                ctx.draw_image(tilecache::get(tilecache::LOGO), V2(274.0, 180.0), 0.0, &color::FIREBRICK, &color::BLACK);
                Fonter::new(ctx)
                    .color(&color::DARKRED)
                    .anchor(Anchor::Bottom)
                    .align(Align::Center)
                    .text(format!("Copyright (C) Risto Saarelma 2011 - 2015\nv{}{}", ::version(), if cfg!(debug_assertions) { " debug" } else { "" }))
                    .draw(V2(320.0, 352.0));
            }
            Event::KeyPressed(Key::Escape) => {
                return Some(Transition::Exit);
            }
            Event::Quit => {
                return Some(Transition::Exit);
            }
            Event::KeyPressed(Key::F12) => { ctx.save_screenshot(&"magog"); }
            Event::KeyPressed(_) => {
                return Some(Transition::Game);
            }
            _ => ()
        }
        None
    }
}
