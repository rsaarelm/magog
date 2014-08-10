use cgmath::point::{Point2};
use calx::engine::{App, Engine, Key};
use calx::color::consts::*;
use calx::engine;
use calx::world::{World, CompProxyMut};
use world::spatial::{Position};
use world::system::{System, EngineLogic, Entity};
use world::fov::Fov;
use world::mapgen::{MapGen};
use world::area::Area;
use world::mobs::{Mobs, MobComp, Mob};
use world::mobs;
use view::worldview::WorldView;
use view::worldview;
use view::tilecache;
use view::tilecache::icon;
use view::main::State;
use view::titlestate::TitleState;

pub struct GameState {
    running: bool,
    world: World<System>,
    camera: Entity,
    in_player_input: bool,
}

impl GameState {
    pub fn new() -> GameState {
        let mut world = World::new(System::new(0));
        GameState {
            running: true,
            world: world.clone(),
            camera: world.new_entity(),
            in_player_input: false,
        }
    }

    fn move(&mut self, dir8: uint) {
        assert!(self.in_player_input);
        let mut player = self.world.player().unwrap();
        let delta = player.smart_move(dir8);
        match delta {
            Some(delta) => {
                // XXX: It's a bit of a wart that we have to explicitly translate the FOV when
                // movement happens. This is needed for non-Euclidean portal maps, but it's not
                // obvious if the code should be supporting those...
                self.get_fov().translate(&delta);
            }
            _ => ()
        }

        if self.world.terrain_at(player.location()).is_exit() {
            self.next_level();
        }

        self.end_turn();
    }

    fn reset_fov(&mut self) {
        self.camera.set_component(Fov::new());
    }

    fn get_fov<'a>(&'a self) -> CompProxyMut<System, Fov> {
        // The camera has to always have the FOV component.
        self.camera.into::<Fov>().unwrap()
    }

    fn camera_to_player(&mut self) {
        // Move camera to player's position and recompute FOV.
        match self.world.player() {
            Some(e) => {
                let loc = e.location();
                self.camera.set_location(loc);
                self.get_fov().update(&self.world, loc, 12);
            }
            _ => ()
        }
    }

    fn next_level(&mut self) {
        self.reset_fov();
        self.world.next_level();
        self.camera_to_player();
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

        self.reset_fov();

        self.world.next_level();
        self.world.player().unwrap().location();
        self.camera_to_player();
    }

    fn key_pressed(&mut self, ctx: &mut Engine, key: Key) {
        if self.in_player_input {
            match key {
                engine::Key1 => { self.next_level(); }
                engine::Key2 => {
                    let mut player = self.world.player().unwrap();
                    let loc = player.location();
                    player.attack(loc);
                }

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
        self.in_player_input = match self.world.player() {
            Some(p) => p.acts_this_frame(),
            None => false
        };

        self.camera_to_player();
        self.world.draw_area(ctx, self.get_fov().deref());

        let _mouse_pos = worldview::draw_mouse(ctx);

        // UI needs player stats to be displayed, so only do it if a player exists.
        match self.world.player() {
            Some(e) => self.draw_ui(ctx, e),
            _ => ()
        }

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

// UI rendering
impl GameState {
    fn health_bar(&self, ctx: &mut Engine, player: Entity) {
        let mob = player.into::<MobComp>().unwrap();
        ctx.set_color(&RED);
        let num_hearts = (mob.max_hp + 1) / 2;
        let solid_hearts = mob.hp / 2;
        let half_heart = (mob.hp % 2) == 1;
        for i in range(0, num_hearts) {
            let pos = Point2::new(i as f32 * 8f32, 8f32);
            let img =
                if i < solid_hearts {
                    icon::HEART
                } else if i == solid_hearts && half_heart {
                    icon::HALF_HEART
                } else {
                    icon::NO_HEART
                };
            ctx.draw_image(&tilecache::get(img), &pos);
        }

        ctx.set_color(&LIGHTSLATEGRAY);
        let num_shards = (mob.armor + 1) / 2;
        let half_shard = (mob.armor % 2) == 1;

        for i in range(0, num_shards) {
            let pos = Point2::new((i + num_hearts) as f32 * 8f32, 8f32);
            let img =
                if i == num_shards - 1 && half_shard {
                    icon::HALF_SHARD
                } else {
                    icon::SHARD
                };
            ctx.draw_image(&tilecache::get(img), &pos);
        }
    }

    fn draw_ui(&self, ctx: &mut Engine, player: Entity) {
        ctx.set_layer(0.100f32);

        self.health_bar(ctx, player);
    }
}
