use std::num::Bounded;
use std::cmp::{min, max};
use collections::hashmap::{HashMap, Keys};
use collections::hashmap::HashSet;
use std::cast;
use cgmath::point::{Point2};
use cgmath::vector::{Vector2};
use cgmath::aabb::{Aabb2};
use calx::rectutil::RectUtil;
use calx::asciimap::{AsciiMap, Cell};
use dijkstra;
use fov::Fov;

// TODO: Figure out how to not require explicit element count.
macro_rules! terrain_data {
    {
        count: $count:expr;
        $($symbol:ident, $name:expr;)*
    } => {
#[deriving(Eq, Clone)]
        pub enum TerrainType {
            $($symbol,)*
        }

        fn terrain_name(t: TerrainType) -> &'static str {
            match t {
                $($symbol => $name,)*
            }
        }

        pub static TERRAINS: [TerrainType, ..$count] = [
            $($symbol,)*
            ];

    }
}

terrain_data! {
    count: 15;

    Void, "void";
    Floor, "floor";
    Water, "water";
    Shallows, "shallows";
    Magma, "magma";
    Downstairs, "stairs down";
    Wall, "wall";
    RockWall, "rock wall";
    Rock, "rock";
    Tree, "tree";
    Grass, "grass";
    Stalagmite, "stalagmite";
    Portal, "portal";
    Door, "door";
    Window, "window";
}


impl TerrainType {
    pub fn from_name(name: &str) -> Option<TerrainType> {
        for &t in TERRAINS.iter() {
            if t.name() == name { return Some(t); }
        }
        None
    }

    pub fn is_wall(self) -> bool {
        match self {
            Wall | RockWall | Rock | Door | Window => true,
            _ => false
        }
    }

    pub fn is_opaque(self) -> bool {
        match self {
            Wall | RockWall | Rock | Door => true,
            _ => false
        }
    }

    pub fn blocks_shot(self) -> bool {
        match self {
            Wall | RockWall | Rock | Tree | Stalagmite | Door => true,
            _ => false
        }
    }

    pub fn is_walkable(self) -> bool {
        match self {
            Floor | Shallows | Grass | Downstairs | Portal | Door => true,
            _ => false
        }
    }

    pub fn name(self) -> &'static str { terrain_name(self) }
}

pub struct Area {
    pub default: TerrainType,
    pub set: HashMap<Location, TerrainType>,
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

    pub fn from_ascii_map(ascii_map: &AsciiMap) -> Area {
        let mut ret = Area::new(
            TerrainType::from_name(ascii_map.default_terrain).unwrap());
        let mut legend = HashMap::new();
        for e in ascii_map.legend.iter() {
            let t = match TerrainType::from_name(e.terrain_type) {
                Some(val) => val,
                None => {
                    println!("Unknown terrain {}", e.terrain_type);
                    continue;
                }
            };
            legend.insert(e.glyph, t);
        }

        for y in range(0, ascii_map.terrain.len()) {
            for (x, glyph) in ascii_map.terrain.get(y).chars().enumerate() {
                let loc = Location::new(
                    (x as int + ascii_map.offset_x) as i8,
                    (y as int + ascii_map.offset_y) as i8);
                match legend.find(&glyph) {
                    Some(&t) => ret.set(loc, t),
                    _ => ()
                };
            }
        }
        // TODO: Spawn handling. (Will probably need to migrate the whole load
        // code to another module eventually, since Area probably won't be
        // doing spawns in any case.)
        ret
    }

    pub fn get(&self, p: Location) -> TerrainType {
        match self.set.find(&p) {
            None => self.default,
            Some(&t) => t
        }
    }

    pub fn set(&mut self, p: Location, t: TerrainType) {
        if t == self.default {
            self.set.remove(&p);
        } else {
            self.set.insert(p, t);
        }
    }

    pub fn defined(&self, p: Location) -> bool {
        self.set.contains_key(&p)
    }

    pub fn dig(&mut self, p: Location) {
        self.set.insert(p, Floor);
    }

    pub fn fill(&mut self, p: Location) {
        self.set(p, self.default);
    }

    pub fn is_opaque(&self, p: Location) -> bool { self.get(p).is_opaque() }

    pub fn is_open(&self, p: Location) -> bool {
        match self.get(p) {
            Floor | Water | Shallows | Magma | Grass | Downstairs | Portal => true,
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

    pub fn build_asciimap(&self) -> AsciiMap {
        // XXX: Spawns will never be added in this version, since Area type
        // doesn't contain dynamic object information at the present.
        AsciiMap::new(self.default.name().to_owned(), self.set.iter().map(
                |(loc, t)|
                (Point2::new(loc.x as int, loc.y as int), Cell::new(t.name().to_owned(), vec!()))))
    }
}

pub static DIRECTIONS6: [Vector2<int>, ..6] = [
    Vector2 { x: -1, y: -1 },
    Vector2 { x:  0, y: -1 },
    Vector2 { x:  1, y:  0 },
    Vector2 { x:  1, y:  1 },
    Vector2 { x:  0, y:  1 },
    Vector2 { x: -1, y:  0 },
];

pub static DIRECTIONS8: [Vector2<int>, ..8] = [
    Vector2 { x: -1, y: -1 },
    Vector2 { x:  0, y: -1 },
    Vector2 { x:  1, y: -1 },
    Vector2 { x:  1, y:  0 },
    Vector2 { x:  1, y:  1 },
    Vector2 { x:  0, y:  1 },
    Vector2 { x: -1, y:  1 },
    Vector2 { x: -1, y:  0 },
];

// TODO: Add third dimension for multiple persistent levels.
#[deriving(Eq, TotalEq, Clone, Hash)]
pub struct Location {
    pub x: i8,
    pub y: i8,
}

impl Location {
    pub fn new(x: i8, y: i8) -> Location { Location { x: x, y: y } }
}

impl<'a> Location {
    pub fn p(&'a self) -> &'a Point2<i8> {
        unsafe {
            cast::transmute(self)
        }
    }
}

impl Add<Vector2<int>, Location> for Location {
    fn add(&self, other: &Vector2<int>) -> Location {
        Location::new(
            (self.x as int + other.x) as i8,
            (self.y as int + other.y) as i8)
    }
}

impl Sub<Location, Vector2<int>> for Location {
    fn sub(&self, other: &Location) -> Vector2<int> {
        Vector2::new((self.x - other.x) as int, (self.y - other.y) as int)
    }
}

// Positions on a virtual infinite 2D chart, which may map to different actual
// Locations.
#[deriving(Eq, TotalEq, Clone, Hash)]
pub struct ChartPos {
    pub x: int,
    pub y: int,
}

impl<'a> ChartPos {
    pub fn new(x: int, y: int) -> ChartPos { ChartPos { x: x, y: y } }

    pub fn from_location(loc: Location) -> ChartPos {
        ChartPos::new(loc.x as int, loc.y as int)
    }

    pub fn to_location(self) -> Location {
        Location::new(self.x as i8, self.y as i8)
    }

    pub fn p(&'a self) -> &'a Point2<int> {
        unsafe {
            cast::transmute(self)
        }
    }
}
