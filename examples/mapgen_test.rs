//! Test map generators by displaying generated text maps in console.

extern crate calx;
extern crate rand;
extern crate world;

use calx::Prefab;
use rand::Rng;
use std::collections::HashMap;
use std::iter::FromIterator;
use world::{Room, Sector};
use world::mapgen::{self, MapGen, Point2D, Size2D, Vault, VaultCell};

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
    type Vault = Room;

    /// Return a random prefab room.
    fn sample_vault<R: Rng>(&mut self, rng: &mut R) -> Self::Vault { rng.gen() }

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

    fn place_vault(&mut self, vault: &Self::Vault, pos: Point2D) {
        let points: Vec<(Point2D, VaultCell)> = vault.get_shape();
        for &(p, c) in &points {
            let p = p + pos.to_vector();
            if !self.terrain.contains_key(&p) {
                continue;
            }
            let c = match c {
                VaultCell::Interior => '.',
                _ => '#',
            };
            self.terrain.insert(p, c);
        }
    }

    fn add_door(&mut self, pos: Point2D) { self.terrain.insert(pos, '+'); }

    fn add_up_stairs(&mut self, pos: Point2D) { self.terrain.insert(pos, '<'); }

    fn add_down_stairs(&mut self, pos: Point2D) { self.terrain.insert(pos, '>'); }
}

fn main() {
    let mut map = TestMap::new();

    let domain: Vec<Point2D> = map.terrain.keys().cloned().collect();
    mapgen::RoomsAndCorridors.dig(&mut rand::thread_rng(), &mut map, domain);

    println!(
        "{}",
        String::from(Prefab::from_iter(
            map.terrain.iter().map(|(p, t)| (p.to_vector(), *t))
        ))
    );
}
