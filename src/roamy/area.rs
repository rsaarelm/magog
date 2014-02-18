use std::hashmap::HashMap;
use std::cast;
use cgmath::point::{Point2};
use cgmath::vector::{Vec2};

#[deriving(Eq)]
pub enum TerrainType {
    Wall,
    Floor,
    Water,
}

pub struct Area {
    set: HashMap<Location, TerrainType>,
}

impl Area {
    pub fn new() -> Area {
        Area {
            set: HashMap::new(),
        }
    }

    pub fn get(&self, p: &Location) -> TerrainType {
        match self.set.find(p) {
            None => Wall,
            Some(&t) => t
        }
    }

    pub fn defined(&self, p: &Location) -> bool {
        self.set.contains_key(p)
    }

    pub fn remove(&mut self, p: &Location) {
        self.set.remove(p);
    }

    pub fn dig(&mut self, p: &Location) {
        self.set.insert(*p, Floor);
    }

    pub fn fill(&mut self, p: &Location) {
        self.set.insert(*p, Wall);
    }

    pub fn is_open(&self, p: &Location) -> bool {
        match self.get(p) {
            Floor | Water => true,
            _ => false
        }
    }

    pub fn is_walkable(&self, p: &Location) -> bool {
        match self.get(p) {
            Floor => true,
            _ => false
        }
    }

    pub fn walk_neighbors(&self, p: &Location) -> ~[Location] {
        let mut ret = ~[];
        for &v in DIRECTIONS.iter() {
            if self.is_walkable(&(p + v)) {
               ret.push(p + v);
            }
        }
        ret
    }
}

pub fn is_solid(t: TerrainType) -> bool {
    t == Wall
}

pub static DIRECTIONS: &'static [Vec2<int>] = &[
    Vec2 { x: -1, y: -1 },
    Vec2 { x:  0, y: -1 },
    Vec2 { x:  1, y:  0 },
    Vec2 { x:  1, y:  1 },
    Vec2 { x:  0, y:  1 },
    Vec2 { x: -1, y:  0 },
];

// Add third dimension for levels.
#[deriving(Eq, IterBytes, Clone, ToStr)]
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
