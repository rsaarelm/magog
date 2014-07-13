use std::collections::hashmap::{HashMap};
use cgmath::vector::{Vector2};
use cgmath::point::{Point2};
use world::terrain::TerrainType;
use world::area::Area;
use calx::world;

pub type Entity = world::Entity<System>;
pub type World = world::World<System>;

pub struct System {
    world: Option<World>,
    pub seed: u32,
    tick: u64,
    pub depth: int,
    pub area: HashMap<Location, TerrainType>,
}

impl world::System for System {
    fn register(&mut self, world: &World) {
        self.world = Some(world.clone());
    }

    fn added(&mut self, _e: &Entity) {}
    fn changed<C>(&mut self, _e: &Entity, _component: Option<&C>) {}
    fn deleted(&mut self, _e: &Entity) {}
}

impl System {
    pub fn new(seed: u32) -> System {
        System {
            world: None,
            seed: seed,
            tick: 0,
            depth: 0,
            area: HashMap::new(),
        }
    }
}

pub trait EngineLogic {
    /// Get the number of the current time frame.
    fn get_tick(&self) -> u64;
    /// Advance to the next time frame.
    fn advance_frame(&mut self);
}

impl EngineLogic for World {
    fn get_tick(&self) -> u64 { self.system().tick }

    fn advance_frame(&mut self) { self.system_mut().tick += 1; }
}

// TODO: Add third dimension for multiple persistent levels.
#[deriving(Eq, PartialEq, Clone, Hash, Show)]
pub struct Location {
    pub x: i8,
    pub y: i8,
}

impl Location {
    pub fn new(x: i8, y: i8) -> Location { Location { x: x, y: y } }
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


// Positions on a virtual infinite 2D chart, which may map to different actual
// Locations.
#[deriving(Eq, PartialEq, Clone, Hash, Show)]
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

    pub fn to_point(self) -> Point2<int> {
        Point2::new(self.x, self.y)
    }
}

impl Add<Vector2<int>, ChartPos> for ChartPos {
    fn add(&self, other: &Vector2<int>) -> ChartPos {
        ChartPos::new(
            (self.x + other.x),
            (self.y + other.y))
    }
}

//pub struct Chart(HashMap<ChartPos, Location>);
pub type Chart = HashMap<ChartPos, Location>;

/// Trait for entities that have a position in space.
pub trait Position {
    fn location(&self) -> Location;
    fn move(&mut self, delta: &Vector2<int>) -> bool;
}

impl Position for Entity {
    fn location(&self) -> Location {
        *self.into::<Location>().unwrap()
    }

    fn move(&mut self, delta: &Vector2<int>) -> bool {
        let new_loc = self.location() + *delta;

        if self.world().is_walkable(new_loc) {
            self.set_component(new_loc);
            return true;
        }

        return false;
    }
}
