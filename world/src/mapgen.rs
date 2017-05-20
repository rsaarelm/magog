use std::collections::BTreeSet;
use std::cmp::max;
use rand::{Rand, Rng, sample};
use euclid::Point2D;
use calx_grid::Dir6;
use calx_alg::{RandomPermutation, RngExt};
use location::Location;
use terraform::{Terraform, TerrainQuery};
use terrain::Terrain;
use onscreen_locations;

pub fn caves<T, R>(world: &mut T, rng: &mut R, start_at: Location, mut cells_to_dig: u32)
    where T: TerrainQuery + Terraform,
          R: Rng
{
    if cells_to_dig == 0 {
        return;
    }

    let mut edge = BTreeSet::new();
    dig(world, &mut edge, start_at);

    // Arbitrary long iteration, should break after digging a sufficient number of cells before
    // this.
    for _ in 0..10000 {
        if edge.is_empty() {
            break;
        }

        let dig_loc = *sample(rng, edge.iter(), 1)[0];

        // Prefer digging narrow corridors, there's an increasing chance to abort the dig when the
        // selected location is in a very open space.
        let adjacent_floors = Dir6::iter()
                                  .filter(|d| world.terrain(dig_loc + **d).is_open())
                                  .count();
        if rng.gen_range(0, max(1, adjacent_floors * 2)) != 0 {
            continue;
        }

        dig(world, &mut edge, dig_loc);
        cells_to_dig -= 1;
        if cells_to_dig == 0 {
            break;
        }
    }

    fn dig<T>(world: &mut T, edge: &mut BTreeSet<Location>, loc: Location)
        where T: TerrainQuery + Terraform
    {
        assert!(world.is_valid_location(loc));
        world.set_terrain(loc, Terrain::Ground);

        edge.remove(&loc);
        for &d in Dir6::iter() {
            let edge_loc = loc + d;

            if world.is_valid_location(edge_loc) && world.terrain(edge_loc).is_hull() {
                edge.insert(edge_loc);
            }
        }
    }
}

pub fn maze<T, R>(world: &mut T, rng: &mut R, sparseness: usize)
    where T: TerrainQuery + Terraform,
          R: Rng
{
    use Terrain::*;

    fn dead_ends<'a, T, I>(world: &T, i: &'a mut I) -> Vec<Location>
        where T: TerrainQuery,
              I: Iterator<Item = &'a Location>
    {
        i.filter(|&x| {
             DIRS4.iter()
                  .filter(|&d| world.terrain(*x + *d).is_open())
                  .count() == 1
         })
         .cloned()
         .collect()
    }

    let mut retries_left = 1000;

    static DIRS4: [Dir6; 4] = [Dir6::Northeast, Dir6::Southeast, Dir6::Southwest, Dir6::Northwest];

    let mut unvisited: BTreeSet<Location> = onscreen_locations()
                                                .iter()
                                                .filter(|p| p.x % 2 == 0 && p.y % 2 == 0)
                                                .map(|p| Location::new(p.x as i8, p.y as i8, 0))
                                                .collect();
    let mut visited = BTreeSet::new();

    let mut current = *sample(rng, unvisited.iter(), 1)[0];
    unvisited.remove(&current);
    visited.insert(current);

    while !unvisited.is_empty() {
        let dig_options: Vec<Dir6> = DIRS4.iter()
                                          .cloned()
                                          .filter(|d| unvisited.contains(&(current + *d + *d)))
                                          .collect();
        if dig_options.is_empty() {
            // Dead end, jump to an earlier pos.
            current = *sample(rng, visited.iter(), 1)[0];

            // If this happens too many times, abort.
            retries_left -= 1;
            if retries_left <= 0 {
                return;
            }

            continue;
        }

        let dig_dir = *sample(rng, dig_options.iter(), 1)[0];

        world.set_terrain(current + dig_dir, Corridor);
        world.set_terrain(current + dig_dir + dig_dir, Corridor);
        current = current + dig_dir + dig_dir;
        unvisited.remove(&current);
        visited.insert(current);
    }

    for _ in 0..sparseness {
        for loc in dead_ends(world, &mut visited.iter()) {
            visited.remove(&loc);
            // TODO: Don't hardcode terrain type in 'undig' operation.
            world.set_terrain(loc, Rock);
            for &dir in &DIRS4 {
                world.set_terrain(loc + dir, Rock);
            }
        }
    }

    // Connect simple loops.
    //
    // FIXME: This is too weak, mazes are still not very loopy.
    for loc in dead_ends(world, &mut visited.iter()) {
        for &dir in &DIRS4 {
            if world.terrain(loc + dir + dir).is_open() && rng.with_chance(0.7) {
                world.set_terrain(loc + dir, Corridor);
            }
        }
    }
}

pub fn rooms<T, R>(world: &mut T, rng: &mut R)
    where T: TerrainQuery + Terraform,
          R: Rng + 'static
{
    // Accept the first room site with badness below this threshold.
    static GOOD_ENOUGH_BADNESS: f32 = 15.0;
    static UNACCEPTABLE_BADNESS: f32 = 350.0;
    let mut failures_left = 20;
    let mut rooms_left = rng.gen_range(4, 9);

    maze(world, rng, 8);

    while rooms_left > 0 && failures_left > 0 {
        let room = EmptyRoom::rand(rng);

        let mut best_site = None;
        let sites = onscreen_locations();

        for site in RandomPermutation::new(rng, sites.len())
                        .map(|idx| Location::new(0, 0, 0) + sites[idx]) {
            if let Some(badness) = room.fit_badness(world, site) {
                if let Some((best, best_badness)) = best_site {
                    if badness < best_badness {
                        best_site = Some((site, badness));
                    }
                } else {
                    best_site = Some((site, badness));
                }
            }

            // Early exit.
            if let Some((best, best_badness)) = best_site {
                if best_badness <= GOOD_ENOUGH_BADNESS {
                    break;
                }
            }
        }

        if let Some((loc, badness)) = best_site {
            if badness > UNACCEPTABLE_BADNESS && failures_left > 1 {
                failures_left -= 1;
            } else {
                room.place(world, loc);
                rooms_left -= 1;
            }
        } else {
            failures_left -= 1;
        }
    }
}

pub trait Room {
    /// Return how well the room will fit in the given location.
    ///
    /// Lower values are better. None means that fit is impossible.
    fn fit_badness<T: TerrainQuery>(&self, w: &T, loc: Location) -> Option<f32>;

    /// Place the room in the given location.
    fn place<T: Terraform + TerrainQuery>(&self, w: &mut T, loc: Location);
}

#[derive(Debug)]
struct EmptyRoom {
    // Room dimensions include walls.
    width: u32,
    height: u32,
}

impl Rand for EmptyRoom {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        EmptyRoom {
            width: rng.gen_range(4, 11),
            height: rng.gen_range(4, 11),
        }
    }
}

impl Room for EmptyRoom {
    fn fit_badness<T: TerrainQuery>(&self, w: &T, loc: Location) -> Option<f32> {
        let mut connected = false;
        let mut badness = 0.0;

        for y in 0..(self.height) {
            for x in 0..(self.width) {
                let is_wall = x == 0 || x == self.width - 1 || y == 0 || y == self.height - 1;
                let is_corner = (x == 0 || x == self.width - 1) && (y == 0 || y == self.height - 1);

                let is_x_wall = (y == 0 || y == self.height - 1) && !is_corner;
                let is_y_wall = (x == 0 || x == self.width - 1) && !is_corner;

                let loc = loc + Point2D::new(x as i32, y as i32);

                if !w.is_valid_location(loc) && !is_wall {
                    // Walls can be placed just outside the screen bounds, but definitely not
                    // floor.
                    return None;
                }

                if is_wall && !is_corner && w.terrain(loc).is_open() {
                    connected = true;
                }

                if w.belongs_to_a_room(loc) {
                    // Rooms overlapping is bad juju.
                    badness += 100.0;
                    continue;
                }

                if is_corner && w.terrain(loc).is_open() {
                    // Please keep corners intact.
                    badness += 100.0;
                    continue;
                }

                if !w.is_untouched(loc) {
                    if is_wall {
                        badness += 1.0;

                        // Non-perpendicular corridor.
                        if is_x_wall &&
                           (!w.is_untouched(loc + Dir6::Northwest) ||
                            !w.is_untouched(loc + Dir6::Southeast)) {
                            badness += 100.0;
                        }

                        if is_y_wall &&
                           (!w.is_untouched(loc + Dir6::Southwest) ||
                            !w.is_untouched(loc + Dir6::Northeast)) {
                            badness += 100.0;
                        }
                    } else {
                        badness += 3.0;
                    }
                }
            }
        }

        if !connected { None } else { Some(badness) }
    }

    fn place<T: Terraform + TerrainQuery>(&self, w: &mut T, loc: Location) {
        use Terrain::*;

        for y in 0..self.height {
            for x in 0..self.width {
                let is_wall = x == 0 || y == 0 || x == self.width - 1 || y == self.height - 1;
                let loc = loc + Point2D::new(x as i32, y as i32);

                let is_corner = (x == 0 || x == self.width - 1) && (y == 0 || y == self.height - 1);

                if is_wall {
                    if w.terrain(loc).is_open() {
                        w.set_terrain(loc, if is_corner { Ground } else { Door });
                    } else {
                        w.set_terrain(loc, Wall);
                    }
                } else {
                    w.set_terrain(loc, Ground);
                }
            }
        }
    }
}
