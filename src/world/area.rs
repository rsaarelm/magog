use std::num::Bounded;
use std::cmp::{min, max};
use collections::hashmap::{HashMap, Keys};
use collections::hashmap::HashSet;
use std::cast;
use cgmath::point::{Point2};
use cgmath::vector::{Vec2};
use cgmath::aabb::{Aabb2};
use calx::rectutil::RectUtil;
use dijkstra;
use fov::Fov;

#[deriving(Eq, Clone)]
pub enum TerrainType {
    Void,
    Floor,
    Water,
    Magma,
    Downstairs,
    Wall,
    RockWall,
    Rock,
    Tree,
    Grass,
    Stalagmite,
    Portal,
}

impl TerrainType {
    pub fn is_wall(self) -> bool {
        match self {
            Wall | RockWall | Rock => true,
            _ => false
        }
    }

    pub fn is_opaque(self) -> bool {
        match self {
            Wall | RockWall | Rock => true,
            _ => false
        }
    }

    pub fn blocks_shot(self) -> bool {
        match self {
            Wall | RockWall | Rock | Tree | Stalagmite => true,
            _ => false
        }
    }

    pub fn is_walkable(self) -> bool {
        match self {
            Floor | Grass | Downstairs | Portal => true,
            _ => false
        }

    }
}

pub struct Area {
    default: TerrainType,
    set: HashMap<Location, TerrainType>,
}

pub type DijkstraMap = HashMap<Location, uint>;

pub fn uphill(map: &DijkstraMap, loc: Location) -> Option<Location> {
    let mut ret = None;
    let mut score = 0;
    for p in DIRECTIONS6.iter().map(|&d| loc + d) {
        let val = map.find(&p);
        match (val, ret) {
            (Some(&s), None) => { score = s; ret = Some(p); },
            (Some(&s), _) => { if s < score { score = s; ret = Some(p); } },
            _ => ()
        };
    }
    ret
}

impl Area {
    pub fn new(default: TerrainType) -> Area {
        Area {
            default: default,
            set: HashMap::new(),
        }
    }

    pub fn get(&self, p: Location) -> TerrainType {
        match self.set.find(&p) {
            None => self.default,
            Some(&t) => t
        }
    }

    pub fn set(&mut self, p: Location, t: TerrainType) {
        self.set.insert(p, t);
    }

    pub fn defined(&self, p: Location) -> bool {
        self.set.contains_key(&p)
    }

    pub fn remove(&mut self, p: Location) {
        self.set.remove(&p);
    }

    pub fn dig(&mut self, p: Location) {
        self.set.insert(p, Floor);
    }

    pub fn fill(&mut self, p: Location) {
        self.set.insert(p, self.default);
    }

    pub fn is_opaque(&self, p: Location) -> bool { self.get(p).is_opaque() }

    pub fn is_open(&self, p: Location) -> bool {
        match self.get(p) {
            Floor | Water | Magma | Grass | Downstairs | Portal => true,
            _ => false
        }
    }

    pub fn iter<'a>(&'a self) -> Keys<'a, Location, TerrainType> {
        self.set.keys()
    }

    pub fn walk_neighbors(&self, p: Location) -> Vec<Location> {
        let mut ret = vec!();
        for &v in DIRECTIONS6.iter() {
            if self.get(p + v).is_walkable() {
               ret.push(p + v);
            }
        }
        ret
    }

    pub fn fully_explored(&self, remembered: &Fov) -> bool {
        // XXX: This won't show maps that have unreachable wall structures
        // buried within other wall tiles as fully explored.
        for &loc in self.cover().iter() {
            if !remembered.contains(loc) {
                return false
            }
        }
        true
    }

    pub fn cover(&self) -> HashSet<Location> {
        // Generate the set of locations that comprise this map. Hit the open
        // cells and their neighbors to get the closest walls.
        let mut ret = HashSet::new();
        for &loc in self.set.keys() {
            ret.insert(loc);
            for &d in DIRECTIONS8.iter() {
                ret.insert(loc + d);
            }
        }
        ret
    }

    pub fn get_bounds(&self) -> Aabb2<int> {
        let mut min_x = Bounded::max_value();
        let mut min_y = Bounded::max_value();
        let mut max_x = Bounded::min_value();
        let mut max_y = Bounded::min_value();
        for &loc in self.iter() {
            min_x = min(min_x, loc.x as int);
            min_y = min(min_y, loc.y as int);
            max_x = max(max_x, loc.x as int);
            max_y = max(max_y, loc.x as int);
        }
        RectUtil::new(min_x, min_y, max_x, max_y)
    }

    pub fn explore_map(&self, remembered: &Fov) -> DijkstraMap {
        let mut goals = vec!();
        for &loc in self.cover().iter() {
            if !remembered.contains(loc) {
                goals.push(loc);
            }
        }

        dijkstra::build_map(goals, |&loc| self.walk_neighbors(loc), 256)
    }
}

pub static DIRECTIONS6: [Vec2<int>, ..6] = [
    Vec2 { x: -1, y: -1 },
    Vec2 { x:  0, y: -1 },
    Vec2 { x:  1, y:  0 },
    Vec2 { x:  1, y:  1 },
    Vec2 { x:  0, y:  1 },
    Vec2 { x: -1, y:  0 },
];

pub static DIRECTIONS8: [Vec2<int>, ..8] = [
    Vec2 { x: -1, y: -1 },
    Vec2 { x:  0, y: -1 },
    Vec2 { x:  1, y: -1 },
    Vec2 { x:  1, y:  0 },
    Vec2 { x:  1, y:  1 },
    Vec2 { x:  0, y:  1 },
    Vec2 { x: -1, y:  1 },
    Vec2 { x: -1, y:  0 },
];

// TODO: Add third dimension for multiple persistent levels.
#[deriving(Eq, TotalEq, Clone, Hash)]
pub struct Location {
    x: i8,
    y: i8,
}

impl Location {
    pub fn new(x: i8, y: i8) -> Location {
        Location { x: x, y: y }
    }
}

impl<'a> Location {
    pub fn p(&'a self) -> &'a Point2<i8> {
        unsafe {
            cast::transmute(self)
        }
    }
}

impl Add<Vec2<int>, Location> for Location {
    fn add(&self, other: &Vec2<int>) -> Location {
        Location::new(
            (self.x as int + other.x) as i8,
            (self.y as int + other.y) as i8)
    }
}

impl Sub<Location, Vec2<int>> for Location {
    fn sub(&self, other: &Location) -> Vec2<int> {
        Vec2::new((self.x - other.x) as int, (self.y - other.y) as int)
    }
}
