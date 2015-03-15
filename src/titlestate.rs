use util::{V2, color, Color, Rgba, Anchor};
use backend::{Key, Event};
use backend::{CanvasUtil, Fonter, Align};
use tilecache;
use world::action;
use ::{Transition, State};

pub struct TitleState {
    tick: usize,
    screenshot_requested: bool,
}

impl TitleState {
    pub fn new() -> TitleState {
        TitleState {
            tick: 0,
            screenshot_requested: false,
        }
    }

}

static FADE_TIME: usize = 64;

impl TitleState {
    fn fade_in<C: Color>(&self, col: &C) -> Rgba {
        let scale = if self.tick < FADE_TIME { self.tick as f32 }
            else { FADE_TIME as f32 } / FADE_TIME as f32;
        let mut rgba = col.to_rgba();
        for i in 0..4 {
            rgba[i] *= scale;
        }

        Color::from_rgba(rgba)
    }

    fn when_faded<C: Color>(&self, col: C) -> C {
        if self.tick < FADE_TIME { Color::from_color(&color::BLACK) } else { col }
    }
}

impl State for TitleState {
    fn process(&mut self, event: Event) -> Option<Transition> {
        self.tick += 1;
        match event {
            Event::Render(ctx) => {
                if self.screenshot_requested {
                    ::screenshot(ctx);
                    self.screenshot_requested = false;
                }

                ctx.draw_image(tilecache::get(tilecache::LOGO), V2(282.0, 180.0), 0.0, &self.fade_in(&color::MEDIUMAQUAMARINE), &color::BLACK);
                Fonter::new(ctx)
                    .color(&self.when_faded(color::DARKCYAN))
                    .anchor(Anchor::Bottom)
                    .align(Align::Center)
                    .text(format!("Copyright (C) Risto Saarelma 2015\nv{}{}", ::version(), if !cfg!(ndebug) { " debug" } else { "" }))
                    .draw(V2(320.0, 352.0));

                Fonter::new(ctx)
                    .color(&self.when_faded(color::DARKCYAN))
                    .anchor(Anchor::TopLeft)
                    .align(Align::Left)
                    .text("N)ew game\nQ)uit".to_string())
                    .draw(V2(280.0, 240.0));
                if action::save_exists() {
                    Fonter::new(ctx)
                        .color(&self.when_faded(color::DARKCYAN))
                        .anchor(Anchor::TopLeft)
                        .align(Align::Left)
                        .text("C)ontinue game".to_string())
                        .draw(V2(280.0, 232.0));
                }
            }
            Event::KeyPressed(Key::Escape) => {
                return Some(Transition::Exit);
            }
            Event::KeyPressed(Key::F12) => { self.screenshot_requested = true; }
            Event::KeyPressed(Key::Q) => {
                return Some(Transition::Exit);
            }
            Event::KeyPressed(Key::N) => {
                action::delete_save();
                return Some(Transition::Game(None));
            }
            Event::KeyPressed(_) => {
                return Some(Transition::Game(None));
            }
            _ => ()
        }
        None
    }
}
