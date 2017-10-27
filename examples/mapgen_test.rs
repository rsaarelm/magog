///!
/// Test map generators by displaying generated text maps in console.
///

extern crate rand;
extern crate calx;
extern crate world;

use calx::Prefab;
use rand::Rng;
use std::collections::HashMap;
use std::iter::FromIterator;
use world::Sector;
use world::mapgen::{self, Point2D, Size2D, DigCavesGen};

struct TestMap {
    terrain: HashMap<Point2D, char>,
}

impl TestMap {
    pub fn new() -> TestMap {
        let mut terrain = HashMap::new();

        let sector = Sector::new(0, 0, 0);
        let sector_origin = sector.origin();
        let mapgen_origin = Point2D::zero();

        terrain.extend(
            sector
                .iter()
                .map(|loc| mapgen_origin + sector_origin.v2_at(loc).unwrap())
                .map(|p| (p, '*')),
        );

        TestMap { terrain }
    }
}

impl mapgen::Dungeon for TestMap {
    type Prefab = Room;

    /// Return a random prefab room.
    fn sample_prefab<R: Rng>(&mut self, rng: &mut R) -> Self::Prefab { Room }

    /// Add a large open continuous region to dungeon.
    fn dig_chamber<I: IntoIterator<Item = Point2D>>(&mut self, area: I) {
        for pos in area {
            self.terrain.insert(pos, '.');
        }
    }

    /// Add a narrow corridor to dungeon.
    fn dig_corridor<I: IntoIterator<Item = Point2D>>(&mut self, path: I) {
        for pos in path {
            self.terrain.insert(pos, '▒');
        }
    }

    fn add_prefab(&mut self, prefab: &Self::Prefab, pos: Point2D) {
        unimplemented!();
    }

    fn add_door(&mut self, pos: Point2D) { self.terrain.insert(pos, '+'); }

    fn add_up_stairs(&mut self, pos: Point2D) { self.terrain.insert(pos, '<'); }

    fn add_down_stairs(&mut self, pos: Point2D) { self.terrain.insert(pos, '>'); }
}

struct Room;
impl mapgen::Prefab for Room {
    fn contains(&self, pos: Point2D) -> bool { false }
    fn can_make_door(&self, pos: Point2D) -> bool { false }
    fn size(&self) -> Size2D { Size2D::new(0, 0) }
}

fn main() {
    let mut map = TestMap::new();

    let gen = DigCavesGen::new(map.terrain.keys().cloned());
    gen.dig(&mut rand::thread_rng(), &mut map);

    println!(
        "{}",
        Prefab::from_iter(map.terrain.iter().map(|(p, t)| (p.to_vector(), *t))).hexmap_display()
    );
}
