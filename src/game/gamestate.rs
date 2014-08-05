use cgmath::point::{Point2};
use calx::engine::{App, Engine, Key};
use calx::color::consts::*;
use calx::engine;
use calx::world::World;
use world::spatial::{Location, Position};
use world::system::{System, EngineLogic};
use world::fov::Fov;
use world::mapgen::{MapGen};
use world::area::Area;
use world::mobs::{Mobs, MobComp, Mob};
use world::mobs;
use view::worldview::WorldView;
use view::worldview;
use game::main::State;
use game::titlestate::TitleState;

pub struct GameState {
    running: bool,
    world: World<System>,
    // TODO: Move Fov into System
    fov: Fov,
    loc: Location,
    in_player_input: bool,
}

impl GameState {
    pub fn new() -> GameState {
        let world = World::new(System::new(0));
        GameState {
            running: true,
            world: world.clone(),
            fov: Fov::new(world.clone()),
            loc: Location::new(0, 3),
            in_player_input: false,
        }
    }

    fn move(&mut self, dir8: uint) {
        assert!(self.in_player_input);
        let mut player = self.world.player().unwrap();
        let delta = player.smart_move(dir8);
        match delta {
            Some(delta) => {
                self.fov.translate(&delta);
                self.loc = self.loc + delta;
            }
            _ => ()
        }

        if self.world.terrain_at(player.location()).is_exit() {
            self.next_level();
        }

        self.end_turn();
    }

    fn next_level(&mut self) {
        let player = self.world.player().unwrap();
        self.fov = Fov::new(self.world.clone());
        self.world.next_level();
        self.loc = player.location();
    }

    fn end_turn(&mut self) {
        self.in_player_input = false;

        self.world.update_mobs();
        self.world.advance_frame();
    }
}

impl App for GameState {
    fn setup(&mut self, _ctx: &mut Engine) {
        let mut e = self.world.new_entity();
        e.set_component(MobComp::new(mobs::Player));

        self.world.next_level();
        self.world.player().unwrap().location();
        self.loc = self.world.player().unwrap().location();
    }

    fn key_pressed(&mut self, ctx: &mut Engine, key: Key) {
        if self.in_player_input {
            match key {
                engine::Key1 => { self.next_level(); }
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
        //self.in_player_input = ai::player_has_turn();
        self.in_player_input = true;

        self.fov.update(self.loc, 12);
        self.world.draw_area(ctx, &self.fov);

        let _mouse_pos = worldview::draw_mouse(ctx);

        ctx.set_color(&WHITE);
        ctx.set_layer(0.100f32);
        ctx.draw_string("Hello, world!", &Point2::new(0f32, 8f32));

        if !self.in_player_input {
            self.end_turn();
        }
    }
}

impl State for GameState {
    fn next_state(&self) -> Option<Box<State>> {
        if !self.running {
            Some(box TitleState::new() as Box<State>)
        } else {
            None
        }
    }
}
