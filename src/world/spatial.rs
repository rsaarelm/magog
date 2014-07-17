use std::collections::hashmap::{HashMap};
use cgmath::vector::{Vector2};
use cgmath::point::{Point2};
use world::system::{Entity};
use world::area::Area;

/// Handle placing and finding entities in space.
pub struct SpatialSystem {
    loc_to_entities: HashMap<Location, Vec<Entity>>,
    entity_to_loc: HashMap<Entity, Location>,
}

impl SpatialSystem {
    pub fn new() -> SpatialSystem {
        SpatialSystem {
            loc_to_entities: HashMap::new(),
            entity_to_loc: HashMap::new(),
        }
    }

    pub fn contains(&self, e: &Entity) -> bool {
        self.entity_to_loc.contains_key(e)
    }

    /// Insert an entity into space.
    pub fn insert(&mut self, e: Entity, loc: Location) {
        assert!(!self.entity_to_loc.contains_key(&e),
            "Inserting an entity already in the space");
        self.entity_to_loc.insert(e.clone(), loc);
        match self.loc_to_entities.find_mut(&loc) {
            Some(v) => { v.push(e); return; }
            _ => ()
        };
        // Didn't return above, that means this location isn't indexed
        // yet and needs a brand new container. (Can't do this in match
        // block because borrows.)
        self.loc_to_entities.insert(loc, vec!(e));
    }

    /// Remove an entity from the space.
    pub fn remove(&mut self, e: &Entity) {
        if !self.entity_to_loc.contains_key(e) { return; }

        let loc = *self.entity_to_loc.find(e).unwrap();
        self.entity_to_loc.remove(e);
        {
            let v = self.loc_to_entities.find_mut(&loc).unwrap();
            assert!(v.len() > 0);
            if v.len() > 1 {
                // More than one entity present, remove this one, keep the
                // rest.
                let idx = v.as_slice().position_elem(e).unwrap();
                v.swap_remove(idx);
                return;
            } else {
                // This was the only entity in the location.
                // Drop the entry for this location from the index.
                // (Need to drop out of scope for borrows reasons)
                assert!(v.get(0) == e);
            }
        }
        // We only end up here if we need to clear the container for the
        // location.
        self.loc_to_entities.remove(&loc);
    }

    /// List entities at a location. Large entities may span multiple
    /// nearby locations, this function will return the entity for each
    /// location its area covers.
    pub fn entities_at(&self, loc: Location) -> Vec<Entity> {
        match self.loc_to_entities.find(&loc) {
            None => vec!(),
            Some(v) => v.clone(),
        }
    }

    /// Return the location of an entity if the entity is present in the space.
    pub fn entity_loc(&self, e: &Entity) -> Option<Location> {
        self.entity_to_loc.find(e).map(|&loc| loc)
    }
}

// TODO: Add third dimension for multiple persistent levels.

/// Unambiguous location in the game world.
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


/// Positions on a virtual infinite 2D chart, which may map to different actual
/// Locations.
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

pub type Chart = HashMap<ChartPos, Location>;

/// Trait for entities that have a position in space.
pub trait Position {
    fn set_location(&mut self, loc: Location);
    fn location(&self) -> Location;
    fn move(&mut self, delta: &Vector2<int>) -> bool;
}

impl Position for Entity {
    fn set_location(&mut self, loc: Location) {
        let old_loc = self.world().system().spatial.entity_loc(self);
        match old_loc {
            // Unchanged location, do nothing.
            Some(l) if l == loc => { return; }
            // Remove self from previous location.
            Some(_) => {
                self.world().system_mut().spatial.remove(self);
            }
            _ => ()
        };

        self.world().system_mut().spatial.insert(self.clone(), loc);
    }

    fn location(&self) -> Location {
        self.world().system().spatial.entity_loc(self).unwrap()
    }

    fn move(&mut self, delta: &Vector2<int>) -> bool {
        let new_loc = self.location() + *delta;

        if self.world().is_walkable(new_loc) {
            self.set_location(new_loc);
            return true;
        }

        return false;
    }
}
