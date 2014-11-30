use std::io::File;
use std::io::fs::PathExtensions;
use std::collections::HashMap;
use calx::color;
use calx::Context;
use calx::Event;
use calx::Key;
use calx::{Fonter, V2};
use world;
use world::action;
use world::action::{Step, Melee};
use world::Dir6;
use world::Dir6::*;
use world::{Entity};
use worldview;
use sprite::{WorldSprites, GibSprite};
use tilecache;
use tilecache::icon;
use msg_queue::MsgQueue;

pub struct GameState {
    world_spr: WorldSprites,
    /// Counters for entities with flashing damage animation.
    damage_timers: HashMap<Entity, uint>,

    // TODO: Probably going to need a general "ongoing activity" system at
    // some point.
    exploring: bool,

    msg: MsgQueue,
}

impl GameState {
    pub fn new(seed: Option<u32>) -> GameState {
        world::init_world(seed);
        GameState {
            world_spr: WorldSprites::new(),
            damage_timers: HashMap::new(),
            exploring: false,
            msg: MsgQueue::new(),
        }
    }

    fn draw_player_ui(&mut self, ctx: &mut Context, player: Entity) {
        let hp = player.hp();
        let max_hp = player.max_hp();

        // Draw heart containers.
        for i in range(0, (max_hp + 1) / 2) {
            let pos = V2(i * 8, 8);
            let idx = if hp >= (i + 1) * 2 { icon::HEART }
                else if hp == i * 2 + 1 { icon::HALF_HEART }
                else { icon::NO_HEART };
            ctx.draw_image(pos, 0.0, tilecache::get(idx), &color::FIREBRICK);
        }
    }

    /// Repaint view, update game world if needed.
    pub fn update(&mut self, ctx: &mut Context) {
        ctx.clear(&color::BLACK);
        let camera = world::camera();

        // Process events
        loop {
            match world::pop_msg() {
                Some(world::Gib(loc)) => {
                    self.world_spr.add(box GibSprite::new(loc));
                }
                Some(world::Damage(entity)) => {
                    self.damage_timers.insert(entity, 2);
                }
                Some(world::Text(txt)) => {
                    self.msg.msg(txt)
                }
                Some(world::Caption(txt)) => {
                    self.msg.caption(txt)
                }
                Some(x) => {
                    println!("Unhandled Msg type {}", x);
                }
                None => break
            }
        }

        worldview::draw_world(&camera, ctx, &self.damage_timers);

        // TODO use FOV for sprite draw.
        self.world_spr.draw(|x| (camera + x).fov_status() == Some(world::Seen), &camera, ctx);
        self.world_spr.update();

        self.msg.draw(ctx);
        if let Some(player) = action::player() {
            self.draw_player_ui(ctx, player);
        }

        let fps = 1.0 / ctx.render_duration;
        let _ = write!(&mut ctx.text_writer(V2(0, 16), 0.1, color::LIGHTGREEN)
                       .set_border(color::BLACK),
                       "FPS {:.0}", fps);

        if action::control_state() == action::ReadyToUpdate {
            action::update();
        }

        if self.exploring {
            if action::control_state() == action::AwaitingInput {
                self.exploring = self.autoexplore();
            }
        }

        self.damage_timers = self.damage_timers.clone().into_iter()
            .filter(|&(_, t)| t > 0u)
            .map(|(e, t)| (e, t - 1))
            .collect();

        self.msg.update();
    }

    pub fn save_game(&self) {
        let save_data = world::save();
        let mut file = File::create(&Path::new("/tmp/magog_save.json"));
        file.write_str(save_data.as_slice()).unwrap();
    }

    pub fn load_game(&mut self) {
        let path = Path::new("/tmp/magog_save.json");
        if !path.exists() { return; }
        let save_data = File::open(&path).read_to_string().unwrap();
        // TODO: Handle failed load nicely.
        world::load(save_data.as_slice()).unwrap();
    }

    fn smart_move(&mut self, dir: Dir6) {
        let player = action::player().unwrap();
        let target_loc = player.location().unwrap() + dir.to_v2();
        if target_loc.has_mobs() {
            action::input(Melee(dir));
        } else {
            action::input(Step(dir));
        }
    }

    fn autoexplore(&mut self) -> bool {
        let player = action::player().unwrap();
        if player.is_threatened() {
            return false;
        }
        if let Some(pathing) = action::autoexplore_map() {
            let loc = player.location().unwrap();
            let steps = pathing.sorted_neighbors(&loc);
            if steps.len() == 0 {
                return false;
            }

            action::input(Step(loc.dir6_towards(steps[0]).unwrap()));
            return true;
        }

        false
    }

    /// Process a player control keypress.
    pub fn process_key(&mut self, key: Key) -> bool {
        if action::control_state() != action::AwaitingInput {
            return false;
        }

        if self.exploring {
            self.exploring = false;
        }

        match key {
            Key::Q | Key::Pad7 => { self.smart_move(NorthWest); }
            Key::W | Key::Pad8 | Key::Up => { self.smart_move(North); }
            Key::E | Key::Pad9 => { self.smart_move(NorthEast); }
            Key::A | Key::Pad1 => { self.smart_move(SouthWest); }
            Key::S | Key::Pad2 | Key::Down => { self.smart_move(South); }
            Key::D | Key::Pad3 => { self.smart_move(SouthEast); }
            Key::F5 => { self.save_game(); }
            Key::F9 => { self.load_game(); }
            Key::X => { self.exploring = true; }
            _ => { return false; }
        }
        return true;
    }

    pub fn process(&mut self, event: Event) -> bool {
        match event {
            Event::Render(ctx) => {
                self.update(ctx);
            }
            Event::KeyPressed(Key::Escape) => {
                return false;
            }
            Event::KeyPressed(k) => {
                self.process_key(k);
            }

            Event::Char(ch) => {
                // TODO: Chars and keypresses in same lookup (use variants?)
                match ch {
                    // Debug
                    '>' => { action::next_level(); }
                    _ => ()
                }
            }

            _ => ()
        }
        true
    }
}
