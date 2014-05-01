use std::cmp::max;
use rand::Rng;
use collections::hashmap::HashSet;

use text::Map2DUtil;
use cgmath::aabb::{Aabb, Aabb2};
use cgmath::point::{Point2};
use world::area::{Area, DIRECTIONS6, Location};
use world::area;

pub trait MapGen {
    fn gen_cave<R: Rng>(&mut self, rng: &mut R, make_exit: bool);
    fn gen_prefab(&mut self, prefab: &str);
}

impl MapGen for Area {
    fn gen_cave<R: Rng>(&mut self, rng: &mut R, make_exit: bool) {
        let center = Location::new(0i8, 0i8);
        let mut edge = HashSet::new();
        let bounds = Aabb2::new(Point2::new(-16i8, -16i8), Point2::new(16i8, 16i8));
        let mut dug = 1;
        self.dig(center);
        for &v in DIRECTIONS6.iter() {
            edge.insert(center + v);
        }

        for _itercount in range(0, 10000) {
            let loc = **rng.sample(edge.iter(), 1).get(0);
            let nfloor = DIRECTIONS6.iter().count(|&v| self.is_open(loc + v));
            assert!(nfloor > 0);

            // Weight digging towards narrow corners.
            if rng.gen_range(0, max(1, nfloor)) != 0 {
                continue;
            }

            self.dig(loc);
            edge.remove(&loc);
            dug += 1;

            for &v in DIRECTIONS6.iter() {
                let p = loc + v;
                if self.get(p) == self.default && bounds.contains(p.p()) {
                    edge.insert(p);
                }
            }

            if dug > 384 { break; }
        }

        if make_exit {
            let down_pos = **rng.sample(edge.iter(), 1).get(0);
            self.set(down_pos, area::Downstairs);
            edge.remove(&down_pos);
        }

        // Depillar
        for &loc in edge.iter() {
            let nfloor = DIRECTIONS6.iter().count(|&v| self.is_open(loc + v));
            assert!(nfloor > 0);
            if nfloor == 6 {
                self.set(loc, area::Stalagmite);
            }
        }
    }

    fn gen_prefab(&mut self, prefab: &str) {
        for (c, x, y) in prefab.chars().map2d() {
            if c == '.' {
                self.set.insert(Location::new(x as i8, y as i8), area::Floor);
            }
            if c == '~' {
                self.set.insert(Location::new(x as i8, y as i8), area::Water);
            }
        }

    }
}
