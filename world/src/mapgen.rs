use calx::{RngExt, hex_disc, HexGeom, hex_neighbors};
use euclid::{self, vec2};
use rand::{self, Rng};
use std::cmp::{PartialOrd, Ord, Ordering};
use std::collections::BTreeSet;
use std::ops::Deref;

pub type Point2D = euclid::Point2D<i32>;
pub type Size2D = euclid::Size2D<i32>;


// We can't put raw Point2D into BTreeSet because they don't implement Ord, and we don't want to
// use HashSet anywhere in the mapgen because mapgen must be totally deterministic for a given rng
// seed and hash containers have unspecified iteration order. A newtype wrapper for getting ord
// lets us work with some minor violence to the code.
#[derive(Copy, Clone, Eq, PartialEq)]
struct OrdPoint(Point2D);

impl PartialOrd for OrdPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.x, self.y).partial_cmp(&(other.x, other.y))
    }
}

impl Ord for OrdPoint {
    fn cmp(&self, other: &Self) -> Ordering { (self.x, self.y).cmp(&(other.x, other.y)) }
}

impl Deref for OrdPoint {
    type Target = Point2D;

    fn deref(&self) -> &Point2D { &self.0 }
}

impl From<Point2D> for OrdPoint {
    fn from(point: Point2D) -> Self { OrdPoint(point) }
}

impl From<OrdPoint> for Point2D {
    fn from(ord_point: OrdPoint) -> Self { ord_point.0 }
}


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
    domain: Vec<OrdPoint>,
}

impl DigCavesGen {
    pub fn new<I: IntoIterator<Item = Point2D>>(domain: I) -> DigCavesGen {
        DigCavesGen { domain: domain.into_iter().map(|x| x.into()).collect() }
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

        let available: BTreeSet<OrdPoint> = self.domain.iter().cloned().collect();

        // Initial cellular automata run, produces okay looking but usually not fully connected
        // cave map.
        let mut dug: BTreeSet<OrdPoint> = self.domain
            .iter()
            .filter(|_| rng.with_chance(INIT_P))
            .cloned()
            .collect();

        for _ in 0..N_ITERS {
            let mut dug2 = dug.clone();

            for &p in &self.domain {
                let n_walls: u32 = hex_disc(Point2D::from(p), 1)
                    .filter(|&p2| p2 != *p)
                    .map(|p2| !dug.contains(&p2.into()) as u32)
                    .sum();
                let n_walls_2: u32 = hex_disc(Point2D::from(p), 2)
                    .filter(|&p2| p2 != *p)
                    .map(|p2| !dug.contains(&p2.into()) as u32)
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
            let p1: OrdPoint = *rand::sample(rng, dug.iter(), 1)[0];
            let p2: OrdPoint = *rand::sample(rng, next_region.iter(), 1)[0];

            dug.extend(next_region.into_iter());
            dug.extend(jaggly_line(rng, &available, p1, p2).into_iter());
        }

        debug_assert!(!dug.is_empty());
        debug_assert!(is_connected(dug.clone()));

        d.dig_chamber(dug.iter().map(|&x| Point2D::from(x)));

        // Pick stair sites
        let upstair_sites: Vec<OrdPoint> = self.domain
            .iter()
            .filter(|&&p| is_upstair_pos(&dug, p))
            .cloned()
            .collect();
        let downstair_sites: Vec<OrdPoint> = self.domain
            .iter()
            .filter(|&&p| is_downstair_pos(&dug, p))
            .cloned()
            .collect();
        d.add_up_stairs(rand::sample(rng, upstair_sites.into_iter(), 1)[0].into());
        d.add_down_stairs(rand::sample(rng, downstair_sites.into_iter(), 1)[0].into());

        fn is_upstair_pos(dug: &BTreeSet<OrdPoint>, pos: OrdPoint) -> bool {
            let dug = |x, y| dug.contains(&OrdPoint::from(Point2D::from(pos) + vec2(x, y)));
            // XXX: Enclosure descriptions, still awful to write...
            dug(1, 1) && !dug(1, 0) && !dug(0, 1) && !dug(-1, -1) && !dug(-1, 0) && !dug(0, -1)
        }

        fn is_downstair_pos(dug: &BTreeSet<OrdPoint>, pos: OrdPoint) -> bool {
            let dug = |x, y| dug.contains(&OrdPoint::from(Point2D::from(pos) + vec2(x, y)));
            // Downstairs needs an extra enclosure behind it for a tile graphics hack.
            dug(-1, -1) && !dug(-1, 0) && !dug(0, -1) && !dug(1, 1) && !dug(1, 0) &&
                !dug(0, 1) && !dug(2, 2) && !dug(2, 1) && !dug(1, 2)
        }
    }
}

fn separate_regions(mut points: BTreeSet<OrdPoint>) -> Vec<BTreeSet<OrdPoint>> {
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

fn flood_fill(mut points: BTreeSet<OrdPoint>, seed: OrdPoint) -> BTreeSet<OrdPoint> {
    let mut ret = BTreeSet::new();
    let mut edge = BTreeSet::new();

    if !points.remove(&seed) {
        return ret;
    }
    edge.insert(seed);

    while !edge.is_empty() {
        // XXX: Is there a set data type that supports 'pop'?
        let pos = *edge.iter().next().unwrap();
        edge.remove(&pos);
        ret.insert(pos);

        for p in hex_neighbors(Point2D::from(pos)) {
            let p = p.into();
            if points.contains(&p) {
                debug_assert!(!ret.contains(&p) && !edge.contains(&p));
                points.remove(&p);
                edge.insert(p);
            }
        }
    }

    ret
}

fn is_connected(points: BTreeSet<OrdPoint>) -> bool { separate_regions(points).len() <= 1 }

// TODO return impl
fn jaggly_line<R: Rng>(
    rng: &mut R,
    available: &BTreeSet<OrdPoint>,
    p1: OrdPoint,
    p2: OrdPoint,
) -> Vec<OrdPoint> {
    let mut ret = Vec::new();
    let mut p = p1;
    ret.push(p);

    while p != p2 {
        let dist = (*p2 - *p).hex_dist();
        let options = hex_neighbors(Point2D::from(p)).filter(|&q| {
            (*p2 - q).hex_dist() < dist && available.contains(&q.into())
        });
        p = rand::sample(rng, options, 1)[0].into();
        ret.push(p);
    }

    ret
}
