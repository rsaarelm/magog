use std::slice::Items;
use cgmath::{Vector, Vector2};
use calx::color::*;
use calx::{Context};
use calx::key;
use world::world::{World, CompProxyMut};
use world::spatial::{Position, Location, DIRECTIONS6};
use world::system::{System, EngineLogic, Entity};
use world::fov::{Fov, Seen};
use world::mapgen::{MapGen};
use world::area::Area;
use world::mobs::{Mobs, MobComp, Mob};
use world::mobs;
use view::worldview;
use view::worldview::{loc_to_view};
use view::tilecache;
use view::drawable::Drawable;
use view::tilecache::icon;
use view::main::State;
use view::titlestate::TitleState;

// TODO: Replace with calx::V2.
fn v2<T>(x: T, y: T) -> Vector2<T> { Vector2::new(x, y) }

trait WorldSprite : Drawable {
    fn update(&mut self);
    fn is_alive(&self) -> bool;
    fn footprint<'a>(&'a self) -> Items<'a, Location>;
}

struct WorldEffects {
    sprites: Vec<Box<WorldSprite + 'static>>,
}

impl WorldEffects {
    pub fn new() -> WorldEffects {
        WorldEffects {
            sprites: vec!(),
        }
    }

    pub fn add(&mut self, spr: Box<WorldSprite + 'static>) {
        self.sprites.push(spr);
    }

    pub fn draw(&self, ctx: &mut Context, center: Location, fov: &Fov) {
        let offset = worldview::loc_to_screen(center, Location::new(0, 0));
        for spr in self.sprites.iter() {
            if spr.footprint()
                .any(|&loc| fov.get(loc) == Some(Seen)) {
                    spr.draw(ctx, &offset);
                }
        }
    }

    pub fn update(&mut self) {
        for spr in self.sprites.mut_iter() { spr.update(); }
        self.sprites.retain(|spr| spr.is_alive());
    }
}

struct BeamSprite {
    p1: Location,
    p2: Location,
    life: int,
    footprint: Vec<Location>,
}

impl BeamSprite {
    pub fn new(p1: Location, p2: Location, life: int) -> BeamSprite {
        BeamSprite {
            p1: p1,
            p2: p2,
            life: life,
            // TODO: Generate intervening points into the footprint. With this
            // footprint you can't see the beam unless either the start or the
            // end point is visible.
            footprint: vec![p1, p2],
        }
    }
}

impl Drawable for BeamSprite {
    fn draw(&self, ctx: &mut Context, offset: &Vector2<f32>) {
        let v1 = loc_to_view(self.p1);
        let v2 = loc_to_view(self.p2);

        // TODO
        /*
        ctx.set_color(&LIME);
        ctx.set_layer(worldview::FX_Z);
        ctx.line_width(3f32);
        ctx.line(&v1.add_v(offset), &v2.add_v(offset));
        */
    }
}

impl WorldSprite for BeamSprite {
    fn update(&mut self) { self.life -= 1; }
    fn is_alive(&self) -> bool { self.life >= 0 }
    fn footprint<'a>(&'a self) -> Items<'a, Location> {
        self.footprint.iter()
    }
}

/// Gameplay screen.
pub struct GameState {
    running: bool,
    world: World<System>,
    in_player_input: bool,
    world_fx: WorldEffects,
}


impl GameState {
    pub fn new() -> GameState {
        GameState {
            running: true,
            world: World::new(System::new(0, Fx::new())),
            in_player_input: false,
            world_fx: WorldEffects::new(),
        }
    }

    fn move(&mut self, dir8: uint) {
        assert!(self.in_player_input);
        let mut player = self.world.player().unwrap();
        player.smart_move(dir8);

        if self.world.terrain_at(player.location()).is_exit() {
            self.next_level();
        }

        self.end_turn();
    }

    fn reset_fov(&mut self) {
        self.world.camera().set_component(Fov::new());
    }

    fn get_fov<'a>(&'a self) -> CompProxyMut<System, Fov> {
        // The camera has to always have the FOV component.
        self.world.camera().into::<Fov>().unwrap()
    }

    fn camera_to_player(&mut self) {
        // Move camera to player's position and recompute FOV.
        match self.world.player() {
            Some(e) => {
                let loc = e.location();
                self.world.camera().set_location(loc);
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

impl /*App for*/ GameState {
    fn setup(&mut self, _ctx: &mut Context) {
        let mut e = self.world.new_entity();
        e.set_component(MobComp::new(mobs::Player));

        self.reset_fov();

        self.world.next_level();
        self.world.player().unwrap().location();
        self.camera_to_player();
    }

    fn key_pressed(&mut self, ctx: &mut Context, key: key::Key) {
        if self.in_player_input {
            match key {
                key::Key1 => { self.next_level(); }
                key::Key2 => {
                    let mut player = self.world.player().unwrap();
                    let loc = player.location();
                    player.attack(loc);
                }
                key::Key3 => {
                    let loc = self.world.player().unwrap().location();
                    for i in range(0, 6) {
                        let p2 = Location::new(
                            loc.x + DIRECTIONS6[i].x as i8 * 6,
                            loc.y + DIRECTIONS6[i].y as i8 * 6,);
                        self.world_fx.add(box BeamSprite::new(loc, p2, 30));
                    }
                }

                key::KeyQ | key::KeyPad7 => { self.move(7); }
                key::KeyW | key::KeyPad8 | key::KeyUp => { self.move(0); }
                key::KeyE | key::KeyPad9 => { self.move(1); }
                key::KeyA | key::KeyPad1 => { self.move(5); }
                key::KeyS | key::KeyPad2 | key::KeyDown => { self.move(4); }
                key::KeyD | key::KeyPad3 => { self.move(3); }
                key::KeyLeft => { self.move(6); }
                key::KeyRight => { self.move(2); }
                _ => (),
            }
        }

        match key {
            key::KeyEscape => { ctx.quit(); }
            key::KeyF12 => { ctx.screenshot("/tmp/shot.png"); }
            _ => (),
        }
    }

    fn draw(&mut self, ctx: &mut Context) {
        self.in_player_input = match self.world.player() {
            Some(p) => p.acts_this_frame(),
            None => false
        };

        self.camera_to_player();
        worldview::draw_area(&self.world, ctx, self.world.camera().location(), self.get_fov().deref());
        self.world_fx.draw(ctx, self.world.camera().location(), self.get_fov().deref());
        self.world_fx.update();

        let _mouse_pos = worldview::draw_mouse(ctx, self.world.camera().location());

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
    fn next_state(&self) -> Option<Box<State + 'static>> {
        if !self.running {
            Some(box TitleState::new() as Box<State + 'static>)
        } else {
            None
        }
    }
}

// UI rendering
impl GameState {
    fn health_bar(&self, ctx: &mut Context, player: Entity) {
        let mob = player.into::<MobComp>().unwrap();
        // TODO
        //ctx.set_color(&RED);
        let num_hearts = (mob.max_hp + 1) / 2;
        let solid_hearts = mob.hp / 2;
        let half_heart = (mob.hp % 2) == 1;
        for i in range(0, num_hearts) {
            let pos = v2(i as f32 * 8f32, 8f32);
            let img =
                if i < solid_hearts {
                    icon::HEART
                } else if i == solid_hearts && half_heart {
                    icon::HALF_HEART
                } else {
                    icon::NO_HEART
                };
            //ctx.draw_image(&tilecache::get(img), &pos);
        }

        //ctx.set_color(&LIGHTSLATEGRAY);
        let num_shards = (mob.armor + 1) / 2;
        let half_shard = (mob.armor % 2) == 1;

        for i in range(0, num_shards) {
            let pos = v2((i + num_hearts) as f32 * 8f32, 8f32);
            let img =
                if i == num_shards - 1 && half_shard {
                    icon::HALF_SHARD
                } else {
                    icon::SHARD
                };
            //ctx.draw_image(&tilecache::get(img), &pos);
        }
    }

    fn draw_ui(&self, ctx: &mut Context, player: Entity) {
        //ctx.set_layer(0.100f32);

        self.health_bar(ctx, player);

        self.world.system().fx.draw(ctx);
    }
}

/// Fire-and-forget on-screen effects.
pub struct Fx {
    // TODO: turn this into a timed queue.
    message: String,
    // TODO: caption
    // TODO: effects
}

impl Fx {
    pub fn new() -> Fx {
        Fx {
            message: "".to_string(),
        }
    }

    pub fn draw(&self, ctx: &mut Context) {
        // TODO
        //ctx.draw_string(self.message.as_slice(), &v2(0f32, 300f32));
    }

    pub fn msg(&mut self, txt: &str) {
        self.message = txt.to_string();
    }

    pub fn caption(&mut self, _txt: &str) {
        unimplemented!();
    }

    pub fn beam(&mut self, _dir6: uint, _length: uint) {
        unimplemented!();
    }
}
