use rand;
use timing::Ticker;
use color::rgb::consts::*;
use cgmath::point::{Point2};
use world::world::{World, Location, DIRECTIONS8};
use world::fov::Fov;
use world::terrain;
use world::mapgen::MapGen;
use view::worldview::WorldView;
use view::worldview;
use engine::{App, Engine, Key, Image};
use engine;

//pub mod game;

/*
static SMART_MOVE_6: &'static [&'static [Vector2<int>]] = &[
    &[DIRECTIONS6[0], DIRECTIONS6[5], DIRECTIONS6[1]],
    &[DIRECTIONS6[1], DIRECTIONS6[0], DIRECTIONS6[2]],
    &[DIRECTIONS6[2], DIRECTIONS6[1], DIRECTIONS6[3]],
    &[DIRECTIONS6[3], DIRECTIONS6[2], DIRECTIONS6[4]],
    &[DIRECTIONS6[4], DIRECTIONS6[3], DIRECTIONS6[5]],
    &[DIRECTIONS6[5], DIRECTIONS6[4], DIRECTIONS6[0]],

    // Sideways move left, right, even column.
    &[DIRECTIONS6[5], DIRECTIONS6[4]],
    &[DIRECTIONS6[1], DIRECTIONS6[2]],

    // Sideways move left, right, odd column.
    &[DIRECTIONS6[4], DIRECTIONS6[5]],
    &[DIRECTIONS6[2], DIRECTIONS6[1]],
];
*/

struct GameApp {
    world: World,
    tiles: Vec<Image>,
    standalone_anim: Ticker,
    fov: Fov,
    loc: Location,
}

impl GameApp {
    pub fn new() -> GameApp {
        GameApp {
            world: World::new(666),
            tiles: vec!(),
            standalone_anim: Ticker::new(0.2f64),
            fov: Fov::new(),
            loc: Location::new(0, 3),
        }
    }
}

impl GameApp {
    fn move(&mut self, dir8: uint) {
        let delta = &DIRECTIONS8[dir8];
        self.fov.translate(delta);
        self.loc = self.loc + *delta;
        self.fov.update(&self.world, self.loc, 12);
    }
}

impl App for GameApp {
    fn setup(&mut self, ctx: &mut Engine) {
        self.tiles = worldview::init_tiles(ctx);
        ctx.set_title("Demogame".to_owned());
        ctx.set_frame_interval(1f64 / 30.0);

        self.world.gen_herringbone(&mut rand::StdRng::new().unwrap());
        self.world.terrain_set(Location::new(0, 0), terrain::Grass);
        self.world.terrain_set(Location::new(1, 0), terrain::Wall);
        self.world.terrain_set(Location::new(1, -1), terrain::Wall);

        self.fov.update(&self.world, self.loc, 12);
    }

    fn key_pressed(&mut self, ctx: &mut Engine, key: Key) {
        /*
        if self.game.player().is_alive() {
            // For the hacked sideways move.
            let column = {
                let player = self.game.player();
                player.loc.p().x - player.loc.p().y
            };
            match key {

                engine::KeyQ | engine::KeyPad7 => { self.game.smart_move(SMART_MOVE_6[5]); },
                engine::KeyW | engine::KeyPad8 => { self.game.smart_move(SMART_MOVE_6[0]); },
                engine::KeyE | engine::PAGEUP => { self.game.smart_move(SMART_MOVE_6[1]); },
                engine::KeyA | engine::END => { self.game.smart_move(SMART_MOVE_6[4]); },
                engine::KeyS | engine::DOWN => { self.game.smart_move(SMART_MOVE_6[3]); },
                engine::KeyD | engine::PAGEDOWN => { self.game.smart_move(SMART_MOVE_6[2]); },

                engine::KeyLeft => { self.game.smart_move(SMART_MOVE_6[ if column % 2 == 0 { 6 } else { 8 }]); },
                engine::KeyRight => { self.game.smart_move(SMART_MOVE_6[ if column % 2 == 0 { 7 } else { 9 }]); },
                engine::KeySpace => { self.game.pass(); },
                _ => (),
            }
        }
        */

        match key {
            engine::KeyQ | engine::KeyPad7 => { self.move(7); }
            engine::KeyW | engine::KeyPad8 | engine::KeyUp => { self.move(0); }
            engine::KeyE | engine::KeyPad9 => { self.move(1); }
            engine::KeyA | engine::KeyPad1 => { self.move(5); }
            engine::KeyS | engine::KeyPad2 | engine::KeyDown => { self.move(4); }
            engine::KeyD | engine::KeyPad3 => { self.move(3); }
            engine::KeyEscape => { ctx.quit(); }
            engine::KeyF12 => { ctx.screenshot("/tmp/shot.png"); }
            _ => (),
        }
    }

    fn draw(&mut self, ctx: &mut Engine) {
        self.world.draw_area(ctx, &self.tiles, &self.fov);

        let _mouse_pos = worldview::draw_mouse(ctx, &self.tiles);

        ctx.set_color(&WHITE);
        ctx.set_layer(0.100f32);
        ctx.draw_string("Hello, world!", &Point2::new(0f32, 8f32));

        /*
        self.game.draw(ctx, self.tiles);


        // Player's gone, assume we're running an attract mode or something.
        if !game.has_player() { break; }

        // Player is dead, run timed animations.
        if !game.player().is_alive() {
            if self.standalone_anim.get() {
                game.update();
            }
        }

        */
    }
}

pub fn main() {
    let mut app = GameApp::new();
    Engine::run(&mut app);
}

/*
pub fn main() {
    let mut app : App<GlRenderer> = App::new(640, 360, "Demogame");
    areaview::init_tiles(&mut app);

    let mut game = Game::new();

    let mut standalone_anim = Ticker::new(0.2f64);

    while app.r.alive {
        game.draw(&mut app);

        let xf = Transform::new(ChartPos::from_location(game.pos));
        let mouse = app.r.get_mouse();
        let cursor_chart_loc = xf.to_chart(&mouse.pos);
        if game.has_player() {
            let _vec = cursor_chart_loc.to_location() - game.pos;
            // TODO: Convert mouse clicks to player input. This needs a
            // slowdown feature to make it work like the keyboard repeat thing,
            // otherwise pressing the mouse button will make the inputs repeat
            // at top speed and make the game unplayable.
        }


        loop {
            // Player's gone, assume we're running an attract mode or something.
            if !game.has_player() { break; }
            if !game.player().is_alive() {
                if standalone_anim.get() {
                    game.update();
                }
            }

            let mut column;

            {
                let player = game.player();
                // For the hacked sideways move.
                column = player.loc.p().x - player.loc.p().y;
            }

            match app.r.pop_key() {
                Some(key) => {
                    if game.player().is_alive() {
                        match key.code {

                            key::Q | key::HOME => { game.smart_move(SMART_MOVE_6[5]); },
                            key::W | key::UP => { game.smart_move(SMART_MOVE_6[0]); },
                            key::E | key::PAGEUP => { game.smart_move(SMART_MOVE_6[1]); },
                            key::A | key::END => { game.smart_move(SMART_MOVE_6[4]); },
                            key::S | key::DOWN => { game.smart_move(SMART_MOVE_6[3]); },
                            key::D | key::PAGEDOWN => { game.smart_move(SMART_MOVE_6[2]); },

                            key::LEFT => { game.smart_move(SMART_MOVE_6[ if column % 2 == 0 { 6 } else { 8 }]); },
                            key::RIGHT => { game.smart_move(SMART_MOVE_6[ if column % 2 == 0 { 7 } else { 9 }]); },
                            key::SPACE => { game.pass(); },
                            _ => (),
                        }
                    }
                    match key.code {
                        key::ESC => { return; },
                        key::F12 => { app.r.screenshot("/tmp/shot.png"); },
                        _ => (),
                    }
                },
                None => { break; }
            }
        }

        app.r.flush();
    }
}
*/
