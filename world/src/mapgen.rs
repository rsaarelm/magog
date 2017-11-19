use calx::{RngExt, hex_disc, HexGeom, hex_neighbors};
use euclid::{self, vec2};
use rand::{self, Rng};
use std::collections::HashSet;

pub type Point2D = euclid::Point2D<i32>;
pub type Size2D = euclid::Size2D<i32>;

pub trait Dungeon {
    type Prefab: Prefab;

    /// Return a random prefab room.
    fn sample_prefab<R: Rng>(&mut self, rng: &mut R) -> Self::Prefab;

    /// Add a large open continuous region to dungeon.
    fn dig_chamber<I: IntoIterator<Item = Point2D>>(&mut self, area: I);

    /// Add a narrow corridor to dungeon.
    fn dig_corridor<I: IntoIterator<Item = Point2D>>(&mut self, path: I);

    fn add_prefab(&mut self, prefab: &Self::Prefab, pos: Point2D);

    fn add_door(&mut self, pos: Point2D);

    fn add_up_stairs(&mut self, pos: Point2D);

    fn add_down_stairs(&mut self, pos: Point2D);
}

pub trait Prefab {
    fn contains(&self, pos: Point2D) -> bool;
    fn can_make_door(&self, pos: Point2D) -> bool;
    fn size(&self) -> Size2D;
}

pub struct DigCavesGen {
    domain: Vec<Point2D>,
}

impl DigCavesGen {
    pub fn new<I: IntoIterator<Item = Point2D>>(domain: I) -> DigCavesGen {
        DigCavesGen { domain: domain.into_iter().collect() }
    }

    pub fn dig<R: Rng, D, P>(&self, rng: &mut R, d: &mut D)
    where
        D: Dungeon<Prefab = P>,
        P: Prefab,
    {
        const INIT_P: f32 = 0.65;
        const N_ITERS: usize = 3;
        const WALL_THRESHOLD: u32 = 3;
        const MINIMAL_REGION_SIZE: usize = 8;

        let available: HashSet<Point2D> = self.domain.iter().cloned().collect();

        // Initial cellular automata run, produces okay looking but usually not fully connected
        // cave map.
        let mut dug: HashSet<Point2D> = self.domain
            .iter()
            .filter(|_| rng.with_chance(INIT_P))
            .cloned()
            .collect();

        for _ in 0..N_ITERS {
            let mut dug2 = dug.clone();

            for &p in &self.domain {
                let n_walls: u32 = hex_disc(p, 1)
                    .filter(|&p2| p2 != p)
                    .map(|p2| !dug.contains(&p2) as u32)
                    .sum();
                let n_walls_2: u32 = hex_disc(p, 2)
                    .filter(|&p2| p2 != p)
                    .map(|p2| !dug.contains(&p2) as u32)
                    .sum();

                // Add a wall if there are either neighboring walls or if the cell is in the
                // centes of a large empty region.
                if n_walls >= WALL_THRESHOLD || n_walls_2 == 0 {
                    dug2.remove(&p);
                } else {
                    dug2.insert(p);
                }
            }

            dug = dug2;
        }

        // Connect separate regions
        let mut regions = separate_regions(dug.clone());
        // Remove uselessly small regions
        regions.retain(|a| a.len() >= MINIMAL_REGION_SIZE);
        // Sort unconnected regions from largest to smallest
        regions.sort_unstable_by(|a, b| b.len().cmp(&a.len()));

        dug = regions.pop().unwrap().into_iter().collect();

        while let Some(next_region) = regions.pop() {
            // Add each secondary region to the dug set, carving a tunnel from dug to its random
            // point.
            // XXX: Might be neater to try to minimize the length of the tunnel to carve
            let p1: Point2D = *rand::sample(rng, dug.iter(), 1)[0];
            let p2: Point2D = *rand::sample(rng, next_region.iter(), 1)[0];

            dug.extend(next_region.into_iter());
            dug.extend(jaggly_line(rng, &available, p1, p2).into_iter());
        }

        debug_assert!(!dug.is_empty());
        debug_assert!(is_connected(dug.clone()));

        d.dig_chamber(dug.clone());

        // Pick stair sites
        let upstair_sites: Vec<Point2D> = self.domain
            .iter()
            .filter(|&&p| is_upstair_pos(&dug, p))
            .cloned()
            .collect();
        let downstair_sites: Vec<Point2D> = self.domain
            .iter()
            .filter(|&&p| is_downstair_pos(&dug, p))
            .cloned()
            .collect();
        d.add_up_stairs(rand::sample(rng, upstair_sites.into_iter(), 1)[0]);
        d.add_down_stairs(rand::sample(rng, downstair_sites.into_iter(), 1)[0]);

        fn is_upstair_pos(dug: &HashSet<Point2D>, pos: Point2D) -> bool {
            // XXX: Enclosure descriptions, still awful to write...
            dug.contains(&(pos + vec2(1, 1))) && !dug.contains(&(pos + vec2(1, 0))) &&
                !dug.contains(&(pos + vec2(0, 1))) &&
                !dug.contains(&(pos + vec2(-1, -1))) &&
                !dug.contains(&(pos + vec2(-1, 0))) &&
                !dug.contains(&(pos + vec2(0, -1)))
        }

        fn is_downstair_pos(dug: &HashSet<Point2D>, pos: Point2D) -> bool {
            // Downstairs needs an extra enclosure behind it for a tile graphics hack.
            dug.contains(&(pos + vec2(-1, -1))) && !dug.contains(&(pos + vec2(-1, 0))) &&
                !dug.contains(&(pos + vec2(0, -1))) &&
                !dug.contains(&(pos + vec2(1, 1))) &&
                !dug.contains(&(pos + vec2(1, 0))) &&
                !dug.contains(&(pos + vec2(0, 1))) &&
                !dug.contains(&(pos + vec2(2, 2))) &&
                !dug.contains(&(pos + vec2(2, 1))) && !dug.contains(&(pos + vec2(1, 2)))
        }
    }
}

fn separate_regions(mut points: HashSet<Point2D>) -> Vec<HashSet<Point2D>> {
    let mut ret = Vec::new();

    while !points.is_empty() {
        let seed = *points.iter().next().unwrap();
        let subset = flood_fill(points.clone(), seed);
        debug_assert!(!subset.is_empty());
        points = points.difference(&subset).cloned().collect();
        ret.push(subset);
    }

    ret
}

fn flood_fill(mut points: HashSet<Point2D>, seed: Point2D) -> HashSet<Point2D> {
    let mut ret = HashSet::new();
    let mut edge = HashSet::new();

    if !points.remove(&seed) {
        return ret;
    }
    edge.insert(seed);

    while !edge.is_empty() {
        // XXX: Is there a set data type that supports 'pop'?
        let pos = *edge.iter().next().unwrap();
        edge.remove(&pos);
        ret.insert(pos);

        for p in hex_neighbors(pos) {
            if points.contains(&p) {
                debug_assert!(!ret.contains(&p) && !edge.contains(&p));
                points.remove(&p);
                edge.insert(p);
            }
        }
    }

    ret
}

fn is_connected(points: HashSet<Point2D>) -> bool { separate_regions(points).len() <= 1 }

// TODO return impl
fn jaggly_line<R: Rng>(rng: &mut R, available: &HashSet<Point2D>, p1: Point2D, p2: Point2D) -> Vec<Point2D> {
    let mut ret = Vec::new();
    let mut p = p1;
    ret.push(p);

    while p != p2 {
        let dist = (p2 - p).hex_dist();
        let options = hex_neighbors(p).filter(|&q| (p2 - q).hex_dist() < dist && available.contains(&q));
        p = rand::sample(rng, options, 1)[0];
        ret.push(p);
    }

    ret
}
