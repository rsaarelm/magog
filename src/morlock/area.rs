use collections::hashmap::HashMap;
use collections::hashmap::HashSet;
use std::cast;
use cgmath::point::{Point2};
use cgmath::vector::{Vec2};
use dijkstra;
use fov::Fov;

#[deriving(Eq)]
pub enum TerrainType {
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
            Floor | Water | Magma | Grass | Downstairs => true,
            _ => false
        }
    }

    pub fn is_walkable(&self, p: Location) -> bool {
        match self.get(p) {
            Floor | Grass | Downstairs => true,
            _ => false
        }
    }

    pub fn walk_neighbors(&self, p: Location) -> ~[Location] {
        let mut ret = ~[];
        for &v in DIRECTIONS6.iter() {
            if self.is_walkable(p + v) {
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

    pub fn explore_map(&self, remembered: &Fov) -> DijkstraMap {
        let mut goals = ~[];
        for &loc in self.cover().iter() {
            if !remembered.contains(loc) {
                goals.push(loc);
            }
        }

        dijkstra::build_map(goals, |&loc| self.walk_neighbors(loc), 256)
    }
}

pub static DIRECTIONS6: &'static [Vec2<int>] = &[
    Vec2 { x: -1, y: -1 },
    Vec2 { x:  0, y: -1 },
    Vec2 { x:  1, y:  0 },
    Vec2 { x:  1, y:  1 },
    Vec2 { x:  0, y:  1 },
    Vec2 { x: -1, y:  0 },
];

pub static DIRECTIONS8: &'static [Vec2<int>] = &[
    Vec2 { x: -1, y: -1 },
    Vec2 { x:  0, y: -1 },
    Vec2 { x:  1, y: -1 },
    Vec2 { x:  1, y:  0 },
    Vec2 { x:  1, y:  1 },
    Vec2 { x:  0, y:  1 },
    Vec2 { x: -1, y:  1 },
    Vec2 { x: -1, y:  0 },
];
// Add third dimension for levels.
#[deriving(Eq, Clone, Hash)]
pub struct Location(Point2<i8>);

impl<'a> Location {
    pub fn p(&'a self) -> &'a Point2<i8> {
        unsafe {
            cast::transmute(self)
        }
    }
}

impl Add<Vec2<int>, Location> for Location {
    fn add(&self, other: &Vec2<int>) -> Location {
        let &Location(p) = self;
        Location(Point2::new(
                (p.x as int + other.x) as i8,
                (p.y as int + other.y) as i8))
    }
}
