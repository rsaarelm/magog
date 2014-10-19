use calx::Rgb;
use calx::color::*;
use {AreaSpec};

/// Data component for mobs.
#[deriving(Clone, Show, Encodable, Decodable)]
pub struct Mob {
    pub max_hp: int,
    pub hp: int,
    pub power: int,
    pub armor: int,
    pub status: int,
}

impl Mob {
    pub fn new(t: MobType) -> Mob {
        let data = SPECS[t as uint];
        let status = if t != Player { status::Asleep as int } else { 0 };
        Mob {
            max_hp: data.power,
            hp: data.power,
            power: data.power,
            armor: 0,
            status: status,
        }
    }

    pub fn has_status(&self, s: status::Status) -> bool {
        self.status as int & s as int != 0
    }

    pub fn add_status(&mut self, s: status::Status) {
        self.status |= s as int;
    }

    pub fn remove_status(&mut self, s: status::Status) {
        self.status &= !(s as int);
    }
}

pub mod intrinsic {
#[deriving(Eq, PartialEq, Clone, Encodable, Decodable)]
pub enum Intrinsic {
    /// Moves 1/3 slower than usual.
    Slow        = 0b1,
    /// Moves 1/3 faster than usual, stacks with Quick status.
    Fast        = 0b10,
    /// Can manipulate objects and doors.
    Hands       = 0b100,
}
}

pub mod status {
#[deriving(Eq, PartialEq, Clone, Encodable, Decodable)]
pub enum Status {
    /// Moves 1/3 slower than usual.
    Slow        = 0b1,
    /// Moves 1/3 faster than usual, stacks with Fast intrinsic.
    Quick       = 0b10,
    /// Mob is inactive until disturbed.
    Asleep      = 0b100,
    /// Mob moves erratically.
    Confused    = 0b1000,
}
}

pub struct MobSpec {
    pub typ: MobType,
    pub name: &'static str,
    pub power: int,
    pub area_spec: AreaSpec,
    pub sprite: uint,
    pub color: &'static Rgb,
    pub intrinsics: int,
}

// Intrinsic flag union.
macro_rules! f {
    { $($flag:ident),* } => { 0 $( | intrinsic::$flag as int )* }
}

macro_rules! mob_data {
    {
        count: $count:expr;
        $($symbol:ident: $power:expr, $depth:expr, $biome:ident, $sprite:expr, $color:expr, $flags:expr;)*

    } => {
#[deriving(Eq, PartialEq, Clone, Show, Encodable, Decodable)]
pub enum MobType {
    $($symbol,)*
}

pub static SPECS: [MobSpec, ..$count] = [
    $(MobSpec {
        typ: $symbol,
        name: stringify!($symbol),
        power: $power,
        area_spec: AreaSpec {
            depth: $depth,
            biome: ::$biome,
        },
        sprite: $sprite,
        color: $color,
        intrinsics: $flags,
    },)*
];

// End macro
    }
}

mob_data! {
    count: 10;
//  Symbol   power, depth, biome, sprite, color,        intrinsics
    Player:     6,  -1, Anywhere, 51, &AZURE,            f!();
    Dreg:       1,   1, Anywhere, 72, &OLIVE,            f!(Hands);
    Snake:      1,   1, Overland, 71, &GREEN,            f!();
    Ooze:       3,   3, Dungeon,  77, &LIGHTSEAGREEN,    f!();
    Flayer:     4,   4, Anywhere, 75, &INDIANRED,        f!();
    Ogre:       6,   5, Anywhere, 73, &DARKSLATEGRAY,    f!(Hands);
    Wraith:     8,   6, Dungeon,  74, &HOTPINK,          f!(Hands);
    Octopus:    10,  7, Anywhere, 63, &DARKTURQUOISE,    f!();
    Efreet:     12,  8, Anywhere, 78, &ORANGE,           f!();
    Serpent:    15,  9, Dungeon,  94, &CORAL,            f!();
}


/*
/// Trait for entities that are mobile things.
pub trait Mob {
    fn is_active(&self) -> bool;
    fn acts_this_frame(&self) -> bool;
    fn has_intrinsic(&self, f: intrinsic::Intrinsic) -> bool;
    fn has_status(&self, s: status::Status) -> bool;
    fn add_status(&mut self, s: status::Status);
    fn remove_status(&mut self, s: status::Status);
    fn mob_type(&self) -> MobType;
    fn power(&self) -> int;
    fn update_ai(&mut self);

    /// Try to move the mob in a direction, then try to roll around obstacles
    /// if the direction is blocked.
    fn smart_move(&mut self, dir8: uint) -> Option<Vector2<int>>;

    fn enemy_at(&self, loc: Location) -> Option<Entity>;
    fn attack(&mut self, loc: Location);

    /// Something interesting is happening at location. See if the mob needs to
    /// wake up and investigate.
    fn alert_at(&mut self, loc: Location);
}

impl Mob for Entity {
    fn is_active(&self) -> bool {
        if !self.has::<MobComp>() { return false; }
        if self.has_status(status::Asleep) { return false; }
        return true;
    }

    fn acts_this_frame(&self) -> bool {
        if !self.has::<MobComp>() { return false; }
        if !self.is_active() { return false; }

        // Go through a cycle of 5 phases to get 4 possible speeds.
        // System idea from Jeff Lait.
        let phase = self.world().get_tick() % 5;
        match phase {
            0 => return true,
            1 => return self.has_intrinsic(intrinsic::Fast),
            2 => return true,
            3 => return self.has_status(status::Quick),
            4 => return !self.has_intrinsic(intrinsic::Slow)
                        && !self.has_status(status::Slow),
            _ => fail!("Invalid action phase"),
        }
    }

    fn has_intrinsic(&self, f: intrinsic::Intrinsic) -> bool {
        self.into::<MobComp>().map_or(false,
            |m| SPECS[m.t as uint].intrinsics as int & f as int != 0)
    }

    fn has_status(&self, s: status::Status) -> bool {
        self.into::<MobComp>().map_or(false,
            |m| m.status as int & s as int != 0)
    }

    fn add_status(&mut self, s: status::Status) {
        self.into::<MobComp>().as_mut().map(
            |m| m.status |= s as int);
    }

    fn remove_status(&mut self, s: status::Status) {
        self.into::<MobComp>().as_mut().map(
            |m| m.status &= !(s as int));
    }

    fn mob_type(&self) -> MobType { self.into::<MobComp>().unwrap().t }

    fn power(&self) -> int { self.into::<MobComp>().unwrap().power }

    fn update_ai(&mut self) {
        if self.world().player().is_some() {
            let player = self.world().player().unwrap();
            let pathing = Pathing::at_loc(&self.world(), player.location());

            let mov = pathing.towards_from(self.location());
            match mov {
                Some(loc) => {
                    let move_dir = &self.location().dir6_towards(loc);
                    match self.enemy_at(loc) {
                        Some(_) => { self.attack(loc); }
                        _ => { self.offset(move_dir); }
                    }
                    return;
                }
                _ => ()
            }
        }

        // No target, wander around randomly.
        self.offset(rand::task_rng().choose(DIRECTIONS6.as_slice()).unwrap());
    }

    fn smart_move(&mut self, dir8: uint) -> Option<Vector2<int>> {
        static SMART_MOVE: &'static [&'static [Vector2<int>]] = &[
            &[DIRECTIONS6[0], DIRECTIONS6[5], DIRECTIONS6[1]],
            &[DIRECTIONS6[1], DIRECTIONS6[0], DIRECTIONS6[2]],
            &[DIRECTIONS6[2], DIRECTIONS6[1], DIRECTIONS6[3]],
            &[DIRECTIONS6[3], DIRECTIONS6[2], DIRECTIONS6[4]],
            &[DIRECTIONS6[4], DIRECTIONS6[3], DIRECTIONS6[5]],
            &[DIRECTIONS6[5], DIRECTIONS6[4], DIRECTIONS6[0]],

            // Right sideways move zigzag.
            &[DIRECTIONS6[1], DIRECTIONS6[2]],
            &[DIRECTIONS6[2], DIRECTIONS6[1]],

            // Left sideways move zigzag.
            &[DIRECTIONS6[5], DIRECTIONS6[4]],
            &[DIRECTIONS6[4], DIRECTIONS6[5]],
            ];
        // "horizontal" movement is a zig-zag since there's no natural hex axis
        // in that direction. Find out the grid column the mob is on and
        // determine whether to zig or zag based on that.
        let loc = self.location();
        let zag = ((loc.x - loc.y) % 2) as uint;

        let deltas = SMART_MOVE[match dir8 {
                0 => 0,
                1 => 1,
                2 => 6 + zag,
                3 => 2,
                4 => 3,
                5 => 4,
                6 => 8 + zag,
                7 => 5,
                _ => fail!("Invalid dir8"),
            }];

        for delta in deltas.iter() {
            let new_loc = loc + *delta;
            match self.enemy_at(new_loc) {
                Some(_) => {
                    self.attack(new_loc);
                    return None;
                }
                _ => ()
            }
            if self.offset(delta) { return Some(*delta); }
        }

        None
    }

    fn enemy_at(&self, loc: Location) -> Option<Entity> {
        let targs = self.world().mobs_at(loc);
        // Nothing to fight.
        if targs.len() == 0 { return None; }
        // TODO: Alignment check
        Some(targs[0].clone())
    }

    fn attack(&mut self, loc: Location) {
        let p = self.power();
        // No power, can't fight.
        if p == 0 { return; }

        let target = match self.enemy_at(loc) {
            None => return,
            Some(t) => t,
        };

        // Every five points of power is one certain hit.
        let full = p / 5;
        let partial = (p % 5) as f64 / 5.0;

        // TODO: Make some rng utility functions.
        let r = rand::random::<f64>() % 1.0;

        let damage = full + if r < partial { 1 } else { 0 };

        // TODO: A deal_damage method.
        let mut tm = target.into::<MobComp>().unwrap();
        tm.hp -= damage;

        if tm.hp <= 0 {
            if target.mob_type() == Player {
                println!("TODO handle player death");
                tm.hp = tm.max_hp;
            }
            // TODO: Whatever extra stuff we want to do when killing a mob.
            // It's probably a special occasion if it's the player avatar.
            self.world().delete_entity(&target);
        }
    }

    fn alert_at(&mut self, loc: Location) {
        // TODO: More complex logic. Check the square for enemies, don't wake
        // up if enemy is stealthed successfully.
        if self.has_status(status::Asleep) {
            if self.location().dist(loc) < 6 {
                self.remove_status(status::Asleep);

                // XXX: Msging REALLY needs a nicer API.
                // FIXME
                //self.world().system_mut().fx.msg(format!("{} wakes up.", SPECS[self.mob_type() as uint].name).as_slice());
            }
        }
    }
}

/// Game world trait for global creature operations.
pub trait Mobs {
    fn mobs_at(&self, loc: Location) -> Vec<Entity>;
    fn mobs(&self) -> Vec<Entity>;
    fn player(&self) -> Option<Entity>;
    fn player_has_turn(&self) -> bool;
    fn clear_npcs(&mut self);
    fn update_mobs(&mut self);
    fn wake_up_mobs(&mut self);
}

impl Mobs for World {
    fn mobs_at(&self, loc: Location) -> Vec<Entity> {
        self.entities_at(loc).iter().filter(|e| e.has::<MobComp>())
            .map(|e| e.clone()).collect()
    }

    fn mobs(&self) -> Vec<Entity> {
        self.entities().filter(|e| e.has::<MobComp>())
            .map(|e| e.clone()).collect()
    }

    fn player(&self) -> Option<Entity> {
        for e in self.mobs().iter() {
            if e.mob_type() == Player {
                return Some(e.clone());
            }
        }
        None
    }

    fn player_has_turn(&self) -> bool {
        match self.player() {
            Some(p) => p.acts_this_frame(),
            _ => false
        }
    }

    fn clear_npcs(&mut self) {
        for e in self.mobs().iter_mut() {
            if e.mob_type() != Player {
                e.delete();
            }
        }
    }

    fn update_mobs(&mut self) {
        self.wake_up_mobs();

        for mob in self.mobs().iter_mut() {
            if !mob.acts_this_frame() { continue; }
            if mob.mob_type() == Player { continue; }
            mob.update_ai();
        }

        self.advance_frame();
    }

    fn wake_up_mobs(&mut self) {
        if self.player().is_none() { return; }
        let player_fov = self.camera().into::<Fov>().unwrap();
        let player_loc = self.player().unwrap().location();
        for &loc in player_fov.deref().seen_locs() {
            for mob in self.mobs_at(loc).iter_mut() {
                mob.alert_at(player_loc);
            }
        }
    }
}

pub struct Pathing {
    gradient: HashMap<Location, uint>,
}

impl Pathing {
    /// Pathing map towards a single point.
    pub fn at_loc(world: &World, loc: Location) -> Pathing {
        Pathing {
            gradient: dijkstra::build_map(
                vec![loc],
                |&loc| world.open_neighbors(loc, DIRECTIONS6.iter()),
                64),
        }
    }

    fn neighbors(&self, loc: Location) -> Vec<Location> {
        DIRECTIONS6.iter().map(|&d| loc + d).collect()
    }

    pub fn towards_from(&self, current: Location) -> Option<Location> {
        let mut ret = None;
        let mut best = uint::MAX;
        for n in self.neighbors(current).iter() {
            match self.gradient.find(n) {
                Some(&weight) if weight <= best => {
                    ret = Some(*n);
                    best = weight;
                }
                _ => ()
            }
        }
        ret
    }
}
*/
