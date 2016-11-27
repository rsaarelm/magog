use std::collections::BTreeSet;
use std::cmp::max;
use rand::{Rng, sample};
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
