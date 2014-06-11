use timing::Ticker;
use color::rgb::consts::*;
use cgmath::point::{Point2};
use world::world::{World, Location};
use world::fov::Fov;
use world::mapgen::{MapGen};
use world::mobs::{Mobs, Mob};
use world::mobs;
use world::ai::AI;
use world::area::Area;
use world::geomorph::Chunks;
use view::worldview::WorldView;
use view::worldview;
use engine::{App, Engine, Key, Image};
use engine;

struct GameApp {
    world: World,
    tiles: Vec<Image>,
    standalone_anim: Ticker,
    fov: Fov,
    loc: Location,
    in_player_input: bool,
    chunks: Chunks,
}

impl GameApp {
    pub fn new() -> GameApp {
        GameApp {
            world: World::new(666),
            tiles: vec!(),
            standalone_anim: Ticker::new(0.2f64),
            fov: Fov::new(),
            loc: Location::new(0, 3),
            in_player_input: false,
            chunks: Chunks::new(),
        }
    }
}

impl GameApp {
    fn move(&mut self, dir8: uint) {
        assert!(self.in_player_input);
        let player = self.world.player().unwrap();
        let delta = self.world.smart_move(player, dir8);
        match delta {
            Some(delta) => {
                self.fov.translate(&delta);
                self.loc = self.loc + delta;
            }
            _ => ()
        }

        if self.world.terrain_at(self.world.mob(player).loc).is_exit() {
            self.fov = Fov::new();
            self.world.next_level(&self.chunks);
            self.loc = self.world.mob(player).loc;
        }

        self.end_turn();
    }

    fn end_turn(&mut self) {
        self.in_player_input = false;

        self.world.update_mobs();
    }
}

impl App for GameApp {
    fn setup(&mut self, ctx: &mut Engine) {
        self.tiles = worldview::init_tiles(ctx);
        ctx.set_title("Demogame".to_string());
        ctx.set_frame_interval(1f64 / 30.0);

        let player = Mob::new(mobs::Player);
        self.world.insert_mob(player);
        self.world.next_level(&self.chunks);

        self.loc = self.world.mob(self.world.player().unwrap()).loc;
    }

    fn key_pressed(&mut self, ctx: &mut Engine, key: Key) {
        if self.in_player_input {
            match key {
                engine::KeyQ | engine::KeyPad7 => { self.move(7); }
                engine::KeyW | engine::KeyPad8 | engine::KeyUp => { self.move(0); }
                engine::KeyE | engine::KeyPad9 => { self.move(1); }
                engine::KeyA | engine::KeyPad1 => { self.move(5); }
                engine::KeyS | engine::KeyPad2 | engine::KeyDown => { self.move(4); }
                engine::KeyD | engine::KeyPad3 => { self.move(3); }
                engine::KeyLeft => { self.move(6); }
                engine::KeyRight => { self.move(2); }
                engine::Key0 => {
                    // Destroy the player mob, just to test that everything
                    // will keep working if there is no player to be found.
                    let p = self.world.player().unwrap();
                    self.world.remove_mob(p);
                }
                _ => (),
            }
        }

        match key {
            engine::KeyEscape => { ctx.quit(); }
            engine::KeyF12 => { ctx.screenshot("/tmp/shot.png"); }
            _ => (),
        }
    }

    fn draw(&mut self, ctx: &mut Engine) {
        self.in_player_input = self.world.player_has_turn();

        self.fov.update(&self.world, self.loc, 12);
        self.world.draw_area(ctx, &self.tiles, &self.fov);

        let _mouse_pos = worldview::draw_mouse(ctx, &self.tiles);

        ctx.set_color(&WHITE);
        ctx.set_layer(0.100f32);
        ctx.draw_string("Hello, world!", &Point2::new(0f32, 8f32));

        if !self.in_player_input {
            self.end_turn();
        }
    }
}

pub fn main() {
    let mut app = GameApp::new();
    Engine::run(&mut app);
}
