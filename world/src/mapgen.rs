// Map generation, being a convoluted beast of unsavory angles and loathsome tesseractations, is
// best confined here in a private subdimension instead of being fully integrated with the main
// world code.

use calx::{self, hex_disc, hex_neighbors, CellSpace, Dir6, HexGeom, RngExt, WeightedChoice};
use calx::{astar_path, GridNode};
use euclid::{self, TypedRect, point2, size2, vec2};
use rand::{self, Rng};
use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::{BTreeMap, BTreeSet};
use std::iter::FromIterator;
use std::ops::Deref;

pub type Point2D = euclid::TypedPoint2D<i32, CellSpace>;
pub type Size2D = euclid::TypedSize2D<i32, CellSpace>;

type Vector2D = euclid::TypedVector2D<i32, CellSpace>;
type Rect = euclid::TypedRect<i32, CellSpace>;

// We can't put raw Point2D into BTreeSet because they don't implement Ord, and we don't want to
// use HashSet anywhere in the mapgen because mapgen must be totally deterministic for a given rng
// seed and hash containers have unspecified iteration order. A newtype wrapper for getting ord
// lets us work with some minor violence to the code.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
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
    type Vault: Vault;

    /// Return a random vault.
    fn sample_vault<R: Rng>(&mut self, rng: &mut R) -> Self::Vault;

    /// Add a large open continuous region to dungeon.
    fn dig_chamber<I: IntoIterator<Item = Point2D>>(&mut self, area: I);

    /// Add a narrow corridor to dungeon.
    fn dig_corridor<I: IntoIterator<Item = Point2D>>(&mut self, path: I);

    /// Place a vault in the world.
    fn place_vault(&mut self, vault: &Self::Vault, pos: Point2D);

    fn add_door(&mut self, pos: Point2D);

    fn add_up_stairs(&mut self, pos: Point2D);

    fn add_down_stairs(&mut self, pos: Point2D);
}

/// Type specificier for cells in `Vault` shape.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum VaultCell {
    /// Open space inside the vault.
    Interior,
    /// Regular wall around the vault, dig a door in it to connect the vault.
    DiggableWall,
    /// Wall around the vault that needs to stay undug.
    UndiggableWall,
}

/// Rooms and chambers provided by the caller.
///
/// Can include both detailed vaults and procedurally generated simple rooms. The map generator
/// will assume that the player can travel through the inside of the vault area between any valid
/// door position.
pub trait Vault {
    fn get_shape<T: FromIterator<(Point2D, VaultCell)>>(&self) -> T;
}

/// Generic map generator interface.
pub trait MapGen {
    fn dig<R, I, D, V>(&self, rng: &mut R, dungeon: &mut D, domain: I)
    where
        R: Rng,
        D: Dungeon<Vault = V>,
        V: Vault,
        I: IntoIterator<Item = Point2D>;
}

/// Map generator for a system of natural caverns.
pub struct Caves;

impl MapGen for Caves {
    fn dig<R, I, D, V>(&self, rng: &mut R, d: &mut D, domain: I)
    where
        R: Rng,
        D: Dungeon<Vault = V>,
        V: Vault,
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
    fn dig<R, I, D, V>(&self, rng: &mut R, d: &mut D, domain: I)
    where
        R: Rng,
        D: Dungeon<Vault = V>,
        V: Vault,
        I: IntoIterator<Item = Point2D>,
    {
        use self::VaultCell::*;

        let mut domain = DigSpace::new(domain.into_iter().map(|p| p.into()));

        let mut vaults = Vec::new();

        loop {
            let vault = d.sample_vault(rng);
            let vault_shape: VaultShape = (&vault).into();
            let offsets = domain.placement_offsets(&vault_shape);

            if offsets.is_empty() {
                // Can't place anything!
                // TODO: More robust criteria
                // TODO: Retry a few times in case it was just a bad fit
                break;
            }

            let center: Vector2D =
                offsets.iter().fold(vec2(0, 0), |a, &b| a + b) / (offsets.len() as i32);
            let dist = |&v: &Vector2D| ((v - center).square_length() as f32).sqrt();

            // How much extra sampling weight should you get for being further out of center.
            //
            // The center point gets weight 1.0, the most distant point gets weight 1.0 + DIST_WEIGHT.
            const DIST_WEIGHT: f32 = 1.0;

            // Normalize the dist factor based on how far the points are spread.
            let dist_factor =
                DIST_WEIGHT / (1.0 + offsets.iter().map(|v| dist(v) as i32).max().unwrap() as f32);

            let offset = *offsets
                .iter()
                .weighted_choice(rng, |v| 1.0 + dist_factor * dist(v))
                .expect("Failed to sample vault offset");

            d.place_vault(&vault, offset.to_point());
            domain.place(&vault_shape, offset);

            vaults.push((offset, vault_shape));
        }

        if vaults.is_empty() {
            panic!("Couldn't place any rooms!");
        }

        // TODO: Connect vaults with corridors
    }
}

/// Placement determination structure for a vault.
struct VaultShape {
    interior: BTreeSet<OrdPoint>,
    no_dig: BTreeSet<OrdPoint>,
    door_here: BTreeSet<OrdPoint>,
    border: BTreeSet<OrdPoint>,
    bounds: Rect,
}

impl<'a, V: Vault> From<&'a V> for VaultShape {
    fn from(vault: &'a V) -> Self {
        let points: Vec<(Point2D, VaultCell)> = vault.get_shape();

        // Split the vault into multiple masks. These are used to modify the diggable area
        // after vault placement has been found.

        // The interior is dug out of the map.
        let mut interior = BTreeSet::new();
        // No-dig points are not dug out but are removed from diggable space.
        let mut no_dig = BTreeSet::new();
        // These points should get a doorway when they are dug.
        let mut door_here = BTreeSet::new();
        // The border cells, must not hit any dug space when placing vault.
        let mut border = BTreeSet::new();

        for &(p, t) in &points {
            let p: OrdPoint = p.into();
            match t {
                VaultCell::Interior => {
                    interior.insert(p);

                    border.insert(p);
                    // Don't trust the user-provided walls, blob the surroundings of every
                    for p2 in hex_disc(Point2D::from(p), 1) {
                        border.insert(p2.into());
                    }
                }
                VaultCell::DiggableWall => {
                    door_here.insert(p);
                    border.insert(p);
                }
                VaultCell::UndiggableWall => {
                    no_dig.insert(p);
                    border.insert(p);
                }
            }
        }
        border = border.difference(&interior).cloned().collect();

        let bounds = bounding_rect(interior.iter());

        VaultShape {
            interior,
            no_dig,
            door_here,
            border,
            bounds,
        }
    }
}

struct DigSpace {
    /// Diggable area.
    diggable: BTreeSet<OrdPoint>,
    /// Dug out area.
    dug: BTreeSet<OrdPoint>,
    bounds: Rect,
}

impl DigSpace {
    pub fn new<I: IntoIterator<Item = OrdPoint>>(domain: I) -> DigSpace {
        let diggable: BTreeSet<OrdPoint> = domain.into_iter().collect();
        let bounds = bounding_rect(diggable.iter());

        DigSpace {
            diggable,
            dug: BTreeSet::new(),
            bounds,
        }
    }

    /// Find valid placement positions for a vault.
    pub fn placement_offsets(&self, vault: &VaultShape) -> Vec<Vector2D> {
        let mut ret = Vec::new();
        'search: for offset in rect_offsets(&self.bounds, &vault.bounds) {
            // All interior points must hit diggable.
            for p in vault.interior.iter().map(|p| (p.0 + offset).into()) {
                if !self.diggable.contains(&p) {
                    continue 'search;
                }
            }

            // No full-shape must hit dug.
            for p in vault.border.iter().map(|p| (p.0 + offset).into()) {
                if self.dug.contains(&p) {
                    continue 'search;
                }
            }

            // We need the two checks here because there no-dig walls create points that are
            // neither diggable (interior can't be placed there) nor dug (border can still
            // overlap with them).

            // Finally collect the valid positions.
            ret.push(offset);
        }
        ret
    }

    /// Place a vault in the space, reduce diggable area.
    pub fn place(&mut self, vault: &VaultShape, offset: Vector2D) {
        for p in vault.interior.iter() {
            let p = (p.0 + offset).into();
            self.diggable.remove(&p);
            self.dug.insert(p);
        }
        for p in vault.no_dig.iter() {
            let p = (p.0 + offset).into();
            self.diggable.remove(&p);
        }

        self.bounds = bounding_rect(self.diggable.iter());
    }

    /// Try to find a tunnel path between two points.
    pub fn find_tunnel(&self, p1: OrdPoint, p2: OrdPoint) -> Option<Vec<OrdPoint>> {
        let neighbors = |p: &OrdPoint| {
            let mut ret = Vec::with_capacity(8);

            let dist = (p.0 - p2.0).hex_dist() as f32;
            for q in hex_neighbors(p.0) {
                let q = OrdPoint(q);
                let dist = (q.0 - p2.0).hex_dist() as f32;
                if self.dug.contains(&q) {
                    // Moving in dug space, just go wherever you want.
                    ret.push((q, dist));
                } else if !self.diggable.contains(&q) {
                    continue;
                } else {
                    // Diggable, but not dug yet. Can only dig if there are solid walls to both
                    // sides.
                    let dig_dir = Dir6::from_v2(q.0 - p.0);
                    let both_sides_walled = !self.dug
                        .contains(&OrdPoint(p.0 + (dig_dir - 1).to_v2()))
                        && !self.dug.contains(&OrdPoint(p.0 + (dig_dir + 1).to_v2()));
                    if both_sides_walled {
                        ret.push((q, dist));
                    }
                }
            }
            ret
        };

        calx::astar_path(p1, &p2, neighbors)
    }
}

fn rect_offsets(outer: &Rect, inner: &Rect) -> impl Iterator<Item = Vector2D> {
    let w = (outer.size.width - inner.size.width + 1).max(0);
    let h = (outer.size.height - inner.size.height + 1).max(0);
    let offset = outer.origin - inner.origin;

    (0..h).flat_map(move |y| (0..w).map(move |x| vec2(x, y) + offset))
}

/// Convenience function using the local type.
fn bounding_rect<'a, I>(points: I) -> TypedRect<i32, CellSpace>
where
    I: IntoIterator<Item = &'a OrdPoint>,
{
    calx::bounding_rect(points.into_iter().map(|p| &p.0))
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
