use view::tilecache;
use calx;
use calx::{V2};
use calx::color;
/*
use view::tilecache;
use calx::engine::{App, Engine, Key};
use view::titlestate::TitleState;

/// Toplevel game application.
struct GameApp {
    state: Box<State>,
}
*/

pub trait State {
    fn next_state(&self) -> Option<Box<State>>;
}

/*
impl GameApp {
    pub fn new() -> GameApp {
        GameApp {
            state: box TitleState::new(),
        }
    }
}

impl App for GameApp {
    fn setup(&mut self, ctx: &mut Engine) {
        tilecache::init(ctx);
        ctx.set_title("Demogame".to_string());
        ctx.set_frame_interval(1f64 / 30.0);
    }

    fn char_typed(&mut self, ctx: &mut Engine, ch: char) {
        self.state.char_typed(ctx, ch);
    }

    fn key_pressed(&mut self, ctx: &mut Engine, key: Key) {
        self.state.key_pressed(ctx, key);
    }

    fn key_released(&mut self, ctx: &mut Engine, key: Key) {
        self.state.key_released(ctx, key);
    }

    fn draw(&mut self, ctx: &mut Engine) {
        self.state.draw(ctx);
        match self.state.next_state() {
            Some(mut st) => {
                st.setup(ctx);
                self.state = st;
            }
            _ => ()
        }
    }
}

pub fn main() {
    let mut app = GameApp::new();
    Engine::run(&mut app);
}
*/

pub fn main() {
    let mut canvas = calx::Canvas::new();
    tilecache::init(&mut canvas);

    for evt in canvas.run() {
        match evt {
            calx::Render(ctx) => {
                ctx.clear(&color::BLACK);
                let img = ctx.font_image('F').unwrap();
                ctx.draw_image(V2(0, 0), 0.4, tilecache::get(tilecache::tile::AVATAR), &color::ORANGE);
            }
            calx::KeyPressed(calx::key::KeyEscape) => {
                return;
            }
            _ => ()
        }
    }
}
