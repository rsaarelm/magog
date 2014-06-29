use color::consts::*;
use cgmath::point::{Point2};
use world::world::{World, Location};
use world::fov::Fov;
use world::mapgen::{MapGen};
use world::mobs::{Mob};
use world::mobs;
use world::ai;
use world::geomorph::Chunks;
use view::worldview::WorldView;
use view::tilecache;
use view::worldview;
use engine::{App, Engine, Key};
use engine;

struct GameApp {
    fov: Fov,
    loc: Location,
    in_player_input: bool,
    chunks: Chunks,
}

impl GameApp {
    pub fn new() -> GameApp {
        GameApp {
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
        let player = ai::player().unwrap();
        let delta = player.smart_move(dir8);
        match delta {
            Some(delta) => {
                self.fov.translate(&delta);
                self.loc = self.loc + delta;
            }
            _ => ()
        }

        /*
        if self.world.terrain_at(self.world.mob(player).loc).is_exit() {
            self.fov = Fov::new();
            self.world.next_level(&self.chunks);
            self.loc = self.world.mob(player).loc;
        }
        */

        self.end_turn();
    }

    fn end_turn(&mut self) {
        self.in_player_input = false;

        ai::update_mobs();
    }
}

impl App for GameApp {
    fn setup(&mut self, ctx: &mut Engine) {
        tilecache::init(ctx);
        ctx.set_title("Demogame".to_string());
        ctx.set_frame_interval(1f64 / 30.0);

        let player = Mob::new(mobs::Player);
        World::map_mut(|w| {
            w.seed = 666;
            w.insert_mob(player);
            w.next_level(&self.chunks);
        });

        self.loc = ai::player().unwrap().loc();
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
        self.in_player_input = ai::player_has_turn();

        self.fov.update(self.loc, 12);
        World::map_mut(|w| w.draw_area(ctx, &self.fov));

        let _mouse_pos = worldview::draw_mouse(ctx);

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
