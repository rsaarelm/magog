use std::io::File;
use std::io::fs::PathExtensions;

use calx::color;
use calx::Context;
use calx::event;
use calx::key;
use calx::{Fonter, V2};
use world;
use world::action;
use world::action::{Step, Melee};
use world::dir6::*;
use worldview;
use sprite::{WorldSprites, GibSprite};

pub struct GameState {
    world_spr: WorldSprites,
}

impl GameState {
    pub fn new(seed: Option<u32>) -> GameState {
        world::init_world(seed);
        GameState {
            world_spr: WorldSprites::new(),
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
                Some(x) => {
                    println!("Unhandled Msg type {}", x);
                }
                None => break
            }
        }

        worldview::draw_world(&camera, ctx);

        // TODO use FOV for sprite draw.
        self.world_spr.draw(|x| (camera + x).fov_status() == Some(world::Seen), &camera, ctx);
        self.world_spr.update();

        let fps = 1.0 / ctx.render_duration;
        let _ = write!(&mut ctx.text_writer(V2(0, 8), 0.1, color::LIGHTGREEN)
                       .set_border(color::BLACK),
                       "FPS {:.0}", fps);

        if action::control_state() == action::ReadyToUpdate {
            action::update();
        }
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

    /// Process a player control keypress.
    pub fn process_key(&mut self, key: key::Key) -> bool {
        if action::control_state() != action::AwaitingInput {
            return false;
        }

        match key {
            key::KeyQ | key::KeyPad7 => { self.smart_move(NorthWest); }
            key::KeyW | key::KeyPad8 | key::KeyUp => { self.smart_move(North); }
            key::KeyE | key::KeyPad9 => { self.smart_move(NorthEast); }
            key::KeyA | key::KeyPad1 => { self.smart_move(SouthWest); }
            key::KeyS | key::KeyPad2 | key::KeyDown => { self.smart_move(South); }
            key::KeyD | key::KeyPad3 => { self.smart_move(SouthEast); }
            key::KeyF5 => { self.save_game(); }
            key::KeyF9 => { self.load_game(); }
            _ => { return false; }
        }
        return true;
    }

    pub fn process(&mut self, event: event::Event) -> bool {
        match event {
            event::Render(ctx) => {
                self.update(ctx);
            }
            event::KeyPressed(key::KeyEscape) => {
                return false;
            }
            event::KeyPressed(k) => {
                self.process_key(k);
            }
            _ => ()
        }
        true
    }
}
