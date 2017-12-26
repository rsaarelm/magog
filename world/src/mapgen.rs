use calx::{bounding_rect, hex_disc, hex_neighbors, CellSpace, HexGeom, RngExt};
use euclid::{self, TypedRect, point2, size2, vec2};
use rand::{self, Rng};
use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::BTreeSet;
use std::ops::Deref;

pub type Point2D = euclid::TypedPoint2D<i32, CellSpace>;
pub type Size2D = euclid::TypedSize2D<i32, CellSpace>;

type Vector2D = euclid::TypedVector2D<i32, CellSpace>;
type Rect = euclid::TypedRect<i32, CellSpace>;

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

/// Interface to the game state where the map is being generated.
pub trait Dungeon {
    type Vault;

    /// Return a random prefab room.
    fn sample_vault<R: Rng>(&mut self, rng: &mut R) -> Self::Vault;

    /// Add a large open continuous region to dungeon.
    fn dig_chamber<I: IntoIterator<Item = Point2D>>(&mut self, area: I);

    /// Add a narrow corridor to dungeon.
    fn dig_corridor<I: IntoIterator<Item = Point2D>>(&mut self, path: I);

    /// Place a prefab in the world.
    fn add_prefab(&mut self, prefab: &Self::Vault, pos: Point2D);

    fn add_door(&mut self, pos: Point2D);

    fn add_up_stairs(&mut self, pos: Point2D);

    fn add_down_stairs(&mut self, pos: Point2D);
}

#[derive(Copy, Clone, Debug)]
pub enum VaultCell {
    Interior,
    DiggableWall,
    UndiggableWall,
}

/// Generic map generator interface.
pub trait MapGen {
    fn dig<'a, R, I, D, V>(&self, rng: &mut R, dungeon: &'a mut D, domain: I)
    where
        R: Rng,
        D: Dungeon<Vault = V>,
        for<'b> &'b V: IntoIterator<Item = (Point2D, VaultCell)> + 'b,
        I: IntoIterator<Item = Point2D>;
}

/// Map generator for a system of natural caverns.
pub struct Caves;

impl MapGen for Caves {
    fn dig<'a, R, I, D, V>(&self, rng: &mut R, d: &'a mut D, domain: I)
    where
        R: Rng,
        D: Dungeon<Vault = V>,
        for<'b> &'b V: IntoIterator<Item = (Point2D, VaultCell)> + 'b,
        I: IntoIterator<Item = Point2D>,
    {
        let domain: Vec<OrdPoint> = domain.into_iter().map(|p| p.into()).collect();

        const INIT_P: f32 = 0.65;
        const N_ITERS: usize = 3;
        const WALL_THRESHOLD: u32 = 3;
        const MINIMAL_REGION_SIZE: usize = 8;

        let available: BTreeSet<OrdPoint> = domain.iter().cloned().collect();

        // Initial cellular automata run, produces okay looking but usually not fully connected
        // cave map.
        let mut dug: BTreeSet<OrdPoint> = domain
            .iter()
            .filter(|_| rng.with_chance(INIT_P))
            .cloned()
            .collect();

        for _ in 0..N_ITERS {
            let mut dug2 = dug.clone();

            for &p in &domain {
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
        let upstair_sites: Vec<OrdPoint> = domain
            .iter()
            .filter(|&&p| is_upstair_pos(&dug, p))
            .cloned()
            .collect();
        let downstair_sites: Vec<OrdPoint> = domain
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
            dug(-1, -1) && !dug(-1, 0) && !dug(0, -1) && !dug(1, 1) && !dug(1, 0) && !dug(0, 1)
                && !dug(2, 2) && !dug(2, 1) && !dug(1, 2)
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

fn jaggly_line<'a, R: Rng>(
    rng: &'a mut R,
    available: &'a BTreeSet<OrdPoint>,
    p1: OrdPoint,
    p2: OrdPoint,
) -> impl Iterator<Item = OrdPoint> + 'a {
    let mut p = p1;
    (0..)
        .map(move |_| {
            let dist = (*p2 - *p).hex_dist();
            let options = hex_neighbors(Point2D::from(p))
                .filter(|&q| (*p2 - q).hex_dist() < dist && available.contains(&q.into()));
            p = rand::sample(rng, options, 1)[0].into();
            p
        })
        .take_while(move |&p| p != p2)
        .chain(Some(p2))
}

/// Map generator for a dungeon of tunnels and carved rooms.
pub struct RoomsAndCorridors;

impl MapGen for RoomsAndCorridors {
    fn dig<'a, R, I, D, V>(&self, rng: &mut R, d: &'a mut D, domain: I)
    where
        R: Rng,
        D: Dungeon<Vault = V>,
        for<'b> &'b V: IntoIterator<Item = (Point2D, VaultCell)> + 'b,
        I: IntoIterator<Item = Point2D>,
    {
        let domain: Vec<OrdPoint> = domain.into_iter().map(|p| p.into()).collect();
        let mut bounds =
            TypedRect::from_points(&domain.iter().map(|p| p.0).collect::<Vec<Point2D>>());
        // XXX: Correct for unintuitive from_points behavior.
        bounds.size = bounds.size + size2(1, 1);

        loop {
            let vault = d.sample_vault(rng);

            // The full set of points, both the interior and the walls.
            let mut full_set: BTreeSet<OrdPoint> = BTreeSet::new();

            // Only the interior that is actually dug out of the map space.
            let mut interior_set: BTreeSet<OrdPoint> = BTreeSet::new();

            let mut door_positions: BTreeSet<OrdPoint> = BTreeSet::new();

            for (p, t) in vault.into_iter() {
                /*
                let p: OrdPoint = p.clone().into();

                full_set.insert(p);
                match t {
                    VaultCell::Interior => { interior_set.insert(p); }
                    VaultCell::DiggableWall => { door_positions.insert(p); }
                    _ => {}
                }
                */
            }

            unimplemented!();
            /*
            let size = prefab.size();
            let mut shape: BTreeSet<OrdPoint> = BTreeSet::new();
            for y in 0..size.height {
                for x in 0..size.width {
                    let p = point2(x, y);
                    if prefab.contains(p) {
                        shape.insert(p.into());
                    }
                }
            }
            */
        }
    }
}

fn fits_in(piece: &BTreeSet<OrdPoint>, offset: Vector2D, space: &BTreeSet<OrdPoint>) -> bool {
    piece
        .iter()
        .map(|p| OrdPoint(p.0 + offset))
        .all(|p| space.contains(&p))
}

struct Piece {
    points: BTreeSet<OrdPoint>,
    bounds: Rect,
}

impl Piece {
    fn new(points: BTreeSet<OrdPoint>) -> Piece {
        let mut ret = Piece {
            points,
            bounds: Rect::zero(),
        };
        ret.update();
        ret
    }

    fn update(&mut self) {
        self.bounds = bounding_rect(&self.points.iter().map(|p| p.0).collect::<Vec<Point2D>>());
    }
}

fn rect_offsets(outer: &Rect, inner: &Rect) -> impl Iterator<Item = Vector2D> {
    let w = (outer.size.width - inner.size.width + 1).max(0);
    let h = (outer.size.height - inner.size.height + 1).max(0);
    let offset = outer.origin - inner.origin;

    (0..h).flat_map(move |y| (0..w).map(move |x| vec2(x, y) + offset))
}

#[cfg(test)]
mod test {
    use super::*;
    use euclid::{self, rect, vec2};
    type Vector2D = euclid::TypedVector2D<i32, CellSpace>;

    #[test]
    fn test_rect_offsets() {
        assert!(
            rect_offsets(&rect(5, 5, 2, 2), &rect(0, 0, 10, 10))
                .next()
                .is_none()
        );
        assert_eq!(
            vec![vec2(5, 5)],
            rect_offsets(&rect(5, 5, 10, 10), &rect(0, 0, 10, 10)).collect::<Vec<Vector2D>>()
        );
        assert_eq!(
            vec![vec2(5, 5), vec2(6, 5)],
            rect_offsets(&rect(5, 5, 11, 10), &rect(0, 0, 10, 10)).collect::<Vec<Vector2D>>()
        );
        assert_eq!(
            vec![vec2(5, 5), vec2(5, 6)],
            rect_offsets(&rect(5, 5, 10, 11), &rect(0, 0, 10, 10)).collect::<Vec<Vector2D>>()
        );
    }
}
