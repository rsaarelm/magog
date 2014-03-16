use time;
use std::rand;
use std::rand::Rng;
use std::mem;

use cgmath::point::{Point2};
use cgmath::vector::{Vec2};
use cgmath::aabb::{Aabb2};
use color::rgb::consts::*;

use calx::app::App;
use calx::app;
use calx::renderer::Renderer;
use calx::renderer;
use calx::rectutil::RectUtil;

use area::{Location, Area, uphill, DijkstraMap, DIRECTIONS6};
use dijkstra;
use area;
use areaview;
use fov::Fov;
use fov;
use mapgen::MapGen;
use mob;
use mob::Mob;
use transform::Transform;
use sprite;

// XXX: Indiscriminate blob of stuff ahoy
pub struct Game {
    area: ~Area,
    pos: Location,
    seen: ~Fov,
    remembered: ~Fov,
    mobs: ~[Mob],
    player_dijkstra: Option<DijkstraMap>,
    rng: rand::StdRng,
    stop: bool,
    depth: uint,
}

static GUN_RANGE: uint = 8;
static END_LEVEL: uint = 9;

// Smart action in a given direction
pub enum ProbeResult {
    // Free to walk, nothing to attack.
    Move,
    // Can't walk nor attack.
    Blocked,
    // Something next to you, melee attack.
    Melee(uint),
    // Something in the distance, can shoot it.
    Ranged(uint),
}

impl Game {
    pub fn new() -> Game {
        let mut ret = Game {
            area: ~Area::new(area::Rock),
            pos: Location(Point2::new(0i8, 0i8)),
            seen: ~Fov::new(),
            remembered: ~Fov::new(),
            mobs: ~[],
            player_dijkstra: None,
            rng: rand::rng(),
            stop: true,
            depth: 0,
        };
        ret.next_level();
        ret
    }

    pub fn player<'a>(&'a mut self) -> &'a mut Mob {
        for i in self.mobs.mut_iter() {
            if i.t == mob::Player {
                return i;
            }
        }
        fail!("No player mob");
    }

    pub fn player_idx(&self) -> uint {
        for (i, mob) in self.mobs.iter().enumerate() {
            if mob.t == mob::Player {
                return i;
            }
        }
        fail!("No player mob");
    }

    pub fn open_cells(&self) -> ~[Location] {
        let mut ret = ~[];
        for &loc in self.area.iter() {
            if self.area.get(loc).is_walkable() && self.mob_at(loc).is_none() {
                ret.push(loc);
            }
        }
        ret
    }

    pub fn has_player(&self) -> bool {
        for i in self.mobs.iter() {
            if i.t == mob::Player {
                return true;
            }
        }
        false
    }

    pub fn mob_idx_at<'a>(&'a self, loc: Location) -> Option<uint> {
        for (i, mob) in self.mobs.iter().enumerate() {
            if mob.loc == loc && mob.is_alive() {
                return Some(i);
            }
        }
        None
    }

    pub fn drawable_mob_at<'a>(&'a self, loc: Location) -> Option<&'a Mob> {
        let mut ret = None;
        for i in self.mobs.iter() {
            if i.loc == loc {
                // Make sure you show up the live mob if there's a live
                // one and corpses here.
                if i.is_alive() || ret.is_none() { ret = Some(i); }
            }
        }
        ret
    }


    pub fn mob_at<'a>(&'a self, loc: Location) -> Option<&'a Mob> {
        for i in self.mobs.iter() {
            if i.loc == loc && i.is_alive() {
                return Some(i);
            }
        }
        None
    }

    pub fn mob_at_mut<'a>(&'a mut self, loc: Location) -> Option<&'a mut Mob> {
        for i in self.mobs.mut_iter() {
            if i.loc == loc {
                return Some(i);
            }
        }
        None
    }

    pub fn next_level(&mut self) {
        // Player state doesn't persist level-to-level.
        self.mobs = ~[Mob::new(mob::Player, Location(Point2::new(0i8, 0i8)))];
        self.area = ~Area::new(area::Rock);
        self.depth += 1;
        let make_exit = self.depth < END_LEVEL;
        self.area.gen_cave(&mut self.rng, make_exit);

        self.player().loc = Location(Point2::new(0i8, 0i8));

        let sites = self.open_cells();

        let mut spawns = ~[mob::Morlock];
        if self.depth > 3 {
            spawns.push(mob::Centipede);
        }
        if self.depth > 6 {
            spawns.push(mob::BigMorlock);
        }

        for &spawn_loc in self.rng.sample(sites.iter(), 6 + self.depth).iter() {
            // TODO: Minimal depth consideration.
            // TODO: Special spawn logic for the boss.
            let kind = self.rng.choose(spawns);
            self.mobs.push(Mob::new(kind, *spawn_loc));
        }

        if self.depth == END_LEVEL {
            let cells = self.open_cells();
            let site = self.rng.choose(cells);

            self.mobs.push(Mob::new(mob::TimeEater, site));
        }

        self.seen = ~Fov::new();
        self.remembered = ~Fov::new();
    }

    pub fn area_name(&self) -> ~str {
        format!("Floor {}", self.depth)
    }

    pub fn object_name(&self, loc: Location) -> ~str {
        match self.mob_at(loc) {
            Some(mob) => mob.data().name,
            None => ~"",
        }
    }

    pub fn attack(&mut self, _agent_idx: uint, target_idx: uint) {
        // TODO: More interesting logic.
        //self.mobs.remove(target_idx);
        let mob = &mut self.mobs[target_idx];
        mob.hits -= 1;

        if mob.hits > 0 {
            mob.anim_state = mob::Hurt(time::precise_time_s());
        } else {
            mob.anim_state = mob::Dying(time::precise_time_s());
            if mob.t == mob::TimeEater {
                // Killed the end boss! Create a portal.
                self.area.set(mob.loc, area::Portal);
            }
        }
    }

    pub fn is_walkable(&self, loc: Location) -> bool {
        self.area.get(loc).is_walkable() && self.mob_idx_at(loc).is_none()
    }

    pub fn melee_probe_dir(&self, mob_idx: uint, dir: &Vec2<int>) -> ProbeResult {
        let loc = self.mobs[mob_idx].loc + *dir;
        let mut walk_state = Move;

        match self.mob_idx_at(loc) {
            Some(mob_idx) => { return Melee(mob_idx); },
            _ => ()
        };

        if !self.area.get(loc).is_walkable() {
            walk_state = Blocked;
        }

        walk_state
    }

    pub fn probe_dir(&self, dir: &Vec2<int>) -> ProbeResult {
        let mut loc = self.mobs[self.player_idx()].loc + *dir;
        let mut walk_state = Move;

        match self.mob_idx_at(loc) {
            Some(mob_idx) => { return Melee(mob_idx); },
            _ => ()
        };

        if !self.area.get(loc).is_walkable() {
            walk_state = Blocked;
        }

        if self.area.get(loc).blocks_shot() {
            return walk_state;
        }

        for _ in range(1, GUN_RANGE) {
            loc = loc + *dir;

            match self.mob_idx_at(loc) {
                Some(mob_idx) => { return Ranged(mob_idx); },
                _ => ()
            };

            if self.area.get(loc).blocks_shot() {
                return walk_state;
            }
        }

        walk_state
    }

    pub fn walk_neighbors(&self, loc: Location) -> ~[Location] {
        let mut ret = ~[];
        for &v in DIRECTIONS6.iter() {
            if self.is_walkable(loc + v) {
               ret.push(loc + v);
            }
        }
        ret
    }

    pub fn msg(&mut self, _txt: &str) {
        // TODO
    }

    pub fn pass(&mut self) {
        let player_idx = self.player_idx();
        if self.mobs[player_idx].ammo < 6 {
            self.msg("reload");
            self.mobs[player_idx].ammo += 1;
        }
        self.update();
    }

    pub fn win_game(&mut self) {
        self.update();
        self.mobs[self.player_idx()].hits = -666;
        self.mobs[self.player_idx()].anim_state = mob::Invisible;
    }

    pub fn smart_move(&mut self, dirs: &[Vec2<int>]) -> bool {
        let player_idx = self.player_idx();

        for d in dirs.iter() {
            match self.probe_dir(d) {
                Blocked => { continue; },
                Move => {
                    let new_loc = self.player().loc + *d;
                    self.player().loc = new_loc;
                    if self.area.get(new_loc) == area::Downstairs {
                        self.next_level();
                    }
                    if self.area.get(new_loc) == area::Portal {
                        self.win_game();
                    }
                    self.update();
                    return true;
                },
                Melee(mob_idx) => {
                    self.attack(player_idx, mob_idx);
                    self.update();
                    return true;
                },
                Ranged(mob_idx) => {
                    if self.mobs[player_idx].ammo > 0 {
                        self.attack(player_idx, mob_idx);
                        self.mobs[player_idx].ammo -= 1;
                    } else {
                        self.msg("reload");
                        self.mobs[player_idx].ammo += 1;
                    }
                    self.update();
                    return true;
                },
            }
        }
        false
    }

    pub fn mob_move(&mut self, mob_idx: uint, dir: &Vec2<int>) -> bool {
        match self.melee_probe_dir(mob_idx, dir) {
            Blocked => { return false; },
            Move => {
                self.mobs[mob_idx].loc = self.mobs[mob_idx].loc + *dir;
                return true;
            }
            Melee(target_idx) => {
                if self.mobs[target_idx].t == mob::Player {
                    self.attack(mob_idx, target_idx);
                    return true;
                } else {
                    return false;
                }
            },
            _ => { return false; }
        };
    }

    pub fn update(&mut self) {
        if self.has_player() && self.player().is_alive() {
            self.pos = self.player().loc;
            self.player_dijkstra = Some(dijkstra::build_map(
                    ~[self.pos], |&loc| self.walk_neighbors(loc), 256));
        } else {
            self.player_dijkstra = None;
        }

        for i in range(0, self.mobs.len()) {
            if !self.mobs[i].is_alive() || self.mobs[i].t == mob::Player { continue; }

            // Wander around randomly if there's no player to hunt.
            if self.player_dijkstra.is_none() {
                let dir = self.rng.choose(area::DIRECTIONS6);
                self.mob_move(i, &dir);
                continue;
            }

            match uphill(self.player_dijkstra.get_ref(), self.mobs[i].loc) {
                Some(new_loc) => {
                    // TODO: Attack if close enough.
                    match self.mob_idx_at(new_loc) {
                        Some(mob_idx) => {
                            if self.mobs[mob_idx].t == mob::Player {
                                self.attack(i, mob_idx);
                            }
                        },
                        None => {
                            self.mobs[i].loc = new_loc;
                        },
                    };
                },
                None => (),
            }
        }
    }

    pub fn draw<R: Renderer>(&mut self, app: &mut App<R>) {
        let mouse = app.r.get_mouse();
        let xf = Transform::new(self.pos);
        let cursor_chart_loc = xf.to_chart(&mouse.pos);

        let mut tmp_seen = ~fov::fov(self.area, self.pos, 12);
        mem::swap(self.seen, tmp_seen);
        // Move old fov to map memory.
        self.remembered.add(tmp_seen);

        for mob in self.mobs.mut_iter() {
            mob.update_anim();
        }

        areaview::draw_area(self, app);
        app.r.draw_tile(areaview::CURSOR_BOTTOM, &xf.to_screen(cursor_chart_loc), sprite::FLOOR_Z, &FIREBRICK, renderer::ColorKeyDraw);
        app.r.draw_tile(areaview::CURSOR_TOP, &xf.to_screen(cursor_chart_loc), sprite::BLOCK_Z, &FIREBRICK, renderer::ColorKeyDraw);

        let text_zone = Aabb2::new(Point2::new(0.0f32, 200.0f32), Point2::new(240.0f32, 360.0f32));
        app.set_color(&LIGHTSLATEGRAY);

        if self.mobs[self.player_idx()].hits == -666 {
            app.print_words(&text_zone, app::Left, "Morlock Hunter\n\nYou win!");
        } else {
            app.print_words(&text_zone, app::Left, "Morlock Hunter\n\ncontrols\n--------\n\
                            QWE\nASD to move and shoot\nSPACE to rest and reload\n\
                            ESC to exit\n\n7DRL 2014 release");
        }

        app.set_color(&CRIMSON);
        let mut health_str = ~"hits: ";
        for _ in range(0, self.player().hits) { health_str = health_str + "o"; }
        app.print_words(&RectUtil::new(0f32, 0f32, 120f32, 8f32), app::Left, health_str);

        app.set_color(&ROYALBLUE);
        let mut ammo_str = ~"ammo: ";
        for _ in range(0, self.player().ammo) { ammo_str = ammo_str + "|"; }
        app.print_words(&RectUtil::new(0f32, 8f32, 120f32, 16f32), app::Left, ammo_str);

        app.set_color(&CORNFLOWERBLUE);
        app.print_words(&Aabb2::new(Point2::new(260.0f32, 0.0f32), Point2::new(380.0f32, 16.0f32)),
            app::Center, self.object_name(cursor_chart_loc));

        app.set_color(&LIGHTSLATEGRAY);
        app.print_words(&Aabb2::new(Point2::new(560.0f32, 0.0f32), Point2::new(640.0f32, 16.0f32)),
            app::Right, self.area_name());

        if !self.stop {
            if !self.area.fully_explored(self.remembered) {
                let map = self.area.explore_map(self.remembered);
                match uphill(&map, self.pos) {
                    Some(p) => { if self.area.get(p).is_walkable() { self.pos = p; } },
                    None => (),
                }
            }
        }

        if self.area.get(self.pos) == area::Downstairs {
            self.next_level();
        }
    }
}
