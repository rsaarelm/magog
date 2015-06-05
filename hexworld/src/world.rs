use std::convert::{Into};
use std::collections::{HashMap};
use std::f32::consts::{PI};
use tiled;
use calx::{V2, Rgba, Projection, lerp, clamp};
use calx_ecs::{Entity};
use spr::{Spr};
use ::{Terrain};
use cmd;

Ecs! {
    desc: Desc,
    // XXX: All component types must be unique in Ecs, so using unwrapped
    // common types like V2 is a bit iffy.
    pos: V2<i32>,
    mob: Mob,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Desc {
    pub name: String,
    pub icon: Spr,
    pub color: Rgba,
}

impl Desc {
    pub fn new<C: Into<Rgba>>(name: &str, icon: Spr, color: C) -> Desc {
        Desc {
            name: name.to_string(),
            icon: icon,
            color: color.into(),
        }
    }
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Mob {
    /// Goes up after action, when zero can act again.
    pub action_delay: u8,
    pub goals: Vec<Goal>,
    pub anim: Anim,
}

#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum Goal {
    MoveTo(V2<i32>),
    Attack(Entity),
    Guard(Entity),
}

impl Mob {
    pub fn new() -> Mob {
        Mob {
            action_delay: 0,
            goals: Vec::new(),
            anim: Anim::Standstill,
        }
    }
}

#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum Anim {
    /// Inert, no movement.
    Standstill,
    /// Idle anim.
    Alert,
    /// Move away from Tween::other_pos.
    Move(Tween),
    /// Attack towards Tween::other_pos.
    Attack(Tween),
}

impl Anim {
    pub fn get_pos(&self, self_pos: V2<i32>, anim_t: u32, proj: &Projection) -> V2<f32> {
        match self {
            &Anim::Standstill => proj.project(self_pos.map(|x| x as f32)),
            &Anim::Alert => proj.project(self_pos.map(|x| x as f32)) +
                // Bobbing anim
                // XXX: Only bobs 1 pixel despite projection magnitude.
                if (anim_t / 10) % 2 == 0 { V2(0.0, -1.0) } else { V2(0.0, 0.0) },
            &Anim::Move(tween) =>
                // Move from other_pos to self_pos
                proj.project(lerp(self_pos.map(|x| x as f32), tween.other_pos.map(|x| x as f32),
                    1.0 - tween.phase(anim_t))),
            &Anim::Attack(tween) =>
                // Bump towards target at other_pos.
                proj.project(lerp(self_pos.map(|x| x as f32), tween.other_pos.map(|x| x as f32),
                    (tween.phase(anim_t) * PI).sin() / 2.0)),
        }
    }
}

#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Tween {
    pub start_timestamp: u32,
    pub other_pos: V2<i32>,
    pub steps: u8,
}

impl Tween {
    pub fn new(start_timestamp: u32, other_pos: V2<i32>, steps: u8) -> Tween {
        Tween {
            start_timestamp: start_timestamp,
            other_pos: other_pos,
            steps: steps,
        }
    }

    pub fn phase(&self, current_t: u32) -> f32 {
        clamp(0.0, 1.0, (current_t - self.start_timestamp) as f32 / self.steps as f32)
    }
}

pub struct World {
    pub world_t: u32,
    pub anim_t: u32,
    pub ecs: Ecs,
    pub terrain: HashMap<V2<i32>, Terrain>,
}

impl World {
    pub fn new() -> World {
        World {
            world_t: 0,
            anim_t: 0,
            ecs: Ecs::new(),
            terrain: HashMap::new(),
        }
    }

    pub fn load(&mut self, tmx: &tiled::Map) {
        for layer in tmx.layers.first().iter() {
            for (y, row) in layer.tiles.iter().enumerate() {
                for (x, &id) in row.iter().enumerate() {
                    self.terrain.insert(V2(x as i32, y as i32), Terrain::new(id as u8));
                }
            }
        }

        for layer in tmx.layers.get(1).iter() {
            for (y, row) in layer.tiles.iter().enumerate() {
                for (x, &id) in row.iter().enumerate() {
                    let pos = V2(x as i32, y as i32);
                    match id as u8 {
                        0 => {}
                        10 => {self.spawn(Loadout::Enemy, pos);}
                        11 => {self.spawn(Loadout::Player, pos);}
                        _ => panic!("Invalid spawn layer item"),
                    }
                }
            }
        }
    }

    /// Update world when running.
    pub fn update_active(&mut self) {
        self.anim_t += 1;
        self.world_t += 1;

        let actives: Vec<Entity> = self.ecs.iter()
            .filter(|&&e| matches_mask(&self.ecs, e, build_mask!(pos, mob)))
            .cloned().collect();

        for e in actives.into_iter() {
            cmd::update_mob(self, e);
        }
    }

    /// Update world when paused.
    ///
    /// Animations will still run.
    pub fn update_standby(&mut self) {
        self.anim_t += 1;
    }

    pub fn spawn(&mut self, a: Loadout, pos: V2<i32>) -> Entity {
        let e = self.ecs.make();
        for x in loadout(a).iter() {
            x.add_to(&mut self.ecs, e);
        }

        pos.add_to(&mut self.ecs, e);

        e
    }

    pub fn terrain_at(&self, pos: V2<i32>) -> Terrain {
        match self.terrain.get(&pos) {
            Some(&t) => t,
            None => Terrain::Void,
        }
    }
}

fn loadout(a: Loadout) -> Vec<Box<Component>> {
    match a {
        Loadout::Player => loadout! [
            Desc::new("player", Spr::Avatar, "white"),
            Mob::new()
        ],
        Loadout::Enemy => loadout! [
            Desc::new("enemy", Spr::Grunt, "red"),
            Mob::new()
        ],
    }
}

enum Loadout {
    Player,
    Enemy,
}
