use calx::{V2, color, Anchor};
use calx::backend::{Key, Event};
use calx::backend::{Canvas, CanvasUtil, Fonter, Align};
use content::Brush;
use {Screen, ScreenAction};
use gamescreen::GameScreen;

pub struct TitleScreen;

impl TitleScreen {
    pub fn new() -> TitleScreen {
        TitleScreen
    }
}

impl Screen for TitleScreen {
    fn update(&mut self, ctx: &mut Canvas) -> Option<ScreenAction> {
        ctx.draw_image(Brush::Logo.get(0),
                       V2(274.0, 180.0),
                       0.0,
                       color::FIREBRICK,
                       color::BLACK);
        Fonter::new(ctx)
            .color(color::DARKRED)
            .anchor(Anchor::Bottom)
            .align(Align::Center)
            .text(format!("Copyright (C) Risto Saarelma 2011 - 2016\nv{}{}",
                          ::version(),
                          if cfg!(debug_assertions) {
                              " debug"
                          } else {
                              ""
                          }))
            .draw(V2(320.0, 352.0));

        for event in ctx.events().into_iter() {
            match event {
                Event::KeyPress(Key::Escape) => {
                    return Some(ScreenAction::Quit);
                }
                Event::Quit => {
                    return Some(ScreenAction::Quit);
                }
                Event::KeyPress(Key::F12) => {
                    ctx.save_screenshot(&"magog");
                }
                Event::KeyPress(_) => {
                    return Some(ScreenAction::Change(Box::new(GameScreen::new())));
                }
                _ => (),
            }
        }

        None
    }
}
