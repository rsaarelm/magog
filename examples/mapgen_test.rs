//! Test map generators by displaying generated text maps in console.

extern crate calx;
extern crate rand;
extern crate world;

use calx::{CellVector, FromPrefab};
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::str;
use std::str::FromStr;
use world::mapgen::{self, MapGen, Point2D, Vault, VaultCell};
use world::{Room, Sector};

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

    fn add_door(&mut self, pos: Point2D) { self.terrain.insert(pos, '|'); }

    fn add_up_stairs(&mut self, pos: Point2D) { self.terrain.insert(pos, '<'); }

    fn add_down_stairs(&mut self, pos: Point2D) { self.terrain.insert(pos, '>'); }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let seed: u32 = if args.len() > 1 {
        FromStr::from_str(&args[1]).expect("Not a valid rng seed")
    } else {
        rand::thread_rng().gen()
    };
    println!("Seed: {}", seed);
    let mut rng = rand::XorShiftRng::from_seed([seed, seed, seed, seed]);

    let mut map = TestMap::new();

    let domain: Vec<Point2D> = map.terrain.keys().cloned().collect();
    mapgen::RoomsAndCorridors.dig(&mut rng, &mut map, domain);

    let prefab: HashMap<CellVector, char> = map.terrain
        .iter()
        .map(|(p, t)| (p.to_vector(), *t))
        .collect();
    let map_text = String::from_prefab(&prefab);

    let mut new_text = String::new();

    for line in map_text.lines() {
        let mut line = line.to_string();
        let mut bytes: Vec<u8> = line.bytes().collect();
        for i in 1..(bytes.len() - 1) {
            for &(a, b, c) in &[
                ('*', '*', '*'),
                ('#', '#', '#'),
                ('*', '#', '*'),
                ('#', '*', '*'),
                ('|', '#', '#'),
                ('#', '|', '#'),
            ] {
                if bytes[i] == ' ' as u8 && bytes[i - 1] == a as u8 && bytes[i + 1] == b as u8 {
                    bytes[i] = c as u8;
                }
            }
        }
        let _ = writeln!(&mut new_text, "{}", str::from_utf8(&bytes).unwrap());
    }

    println!("{}", new_text);
}
