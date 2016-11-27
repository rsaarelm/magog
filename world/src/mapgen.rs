use std::collections::BTreeSet;
use std::cmp::max;
use rand::{Rand, Rng, sample};
use euclid::Point2D;
use calx_grid::Dir6;
use location::Location;
use terraform::{Terraform, TerrainQuery};
use terrain::Id;

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
        world.set_terrain(loc, Id::Ground as u8);

        edge.remove(&loc);
        for &d in Dir6::iter() {
            let edge_loc = loc + d;

            if world.is_valid_location(edge_loc) && world.terrain(edge_loc).is_hull() {
                edge.insert(edge_loc);
            }
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

struct EmptyRoom {
    // Room dimensions include walls.
    width: u32,
    height: u32,
}

impl Rand for EmptyRoom {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        EmptyRoom {
            width: rng.gen_range(3, 12),
            height: rng.gen_range(3, 12),
        }
    }
}

impl Room for EmptyRoom {
    fn fit_badness<T: TerrainQuery>(&self, w: &T, loc: Location) -> Option<f32> {
        let mut connected = false;
        let mut badness = 0.0;

        for y in 0..(self.height) {
            for x in 0..(self.width) {
                let is_wall = x == 0 || y == 0 || x == self.width - 1 || y == self.height - 1;

                // This is a corner tile that does not connect to the main floor, due to hex map
                // geometry. Breach of this corner will not make the space of the room connected to
                // the map.
                let is_blocked_corner = (x == self.width - 1 && y == 0) ||
                                        (y == self.width - 1 && x == 0);

                let is_corner = is_blocked_corner || (x == 0 && y == 0) ||
                                (x == self.width - 1 && y == self.width - 1);

                let loc = loc + Point2D::new(x as i32, y as i32);

                if !w.is_valid_location(loc) && !is_wall {
                    // Walls can be placed just outside the screen bounds, but definitely not
                    // floor.
                    return None;
                }

                if is_wall && !is_blocked_corner && w.terrain(loc).is_open() {
                    connected = true;
                }

                if w.belongs_to_a_room(loc) {
                    // Rooms overlapping is bad juju.
                    badness += 100.0;
                    continue;
                }

                if is_corner && w.terrain(loc).is_open() {
                    // Please keep corners intact.
                    badness += 40.0;
                    continue;
                }

                if !w.is_untouched(loc) {
                    if is_wall {
                        badness += 1.0;
                    } else {
                        badness += 3.0;
                    }
                }
            }
        }

        if !connected { None } else { Some(badness) }
    }

    fn place<T: Terraform + TerrainQuery>(&self, w: &mut T, loc: Location) {
        use terrain::Id::*;

        for y in 0..self.height {
            for x in 0..self.width {
                let is_wall = x == 0 || y == 0 || x == self.width - 1 || y == self.height - 1;
                let loc = loc + Point2D::new(x as i32, y as i32);

                if is_wall {
                    if w.terrain(loc).is_open() {
                        // TODO: Doors
                        w.set_terrain(loc, Ground as u8);
                    } else {
                        w.set_terrain(loc, Wall as u8);
                    }
                } else {
                    w.set_terrain(loc, Ground as u8);
                }
            }
        }
    }
}
