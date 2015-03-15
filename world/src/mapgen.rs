use num::{Integer};
use rand::Rng;
use util::{V2, Rect};
use geomorph;
use geomorph::{Chunk};
use terrain::TerrainType;
use ::{Biome};

pub fn gen_herringbone<R: Rng, F, G>(
    rng: &mut R, spec: &::AreaSpec, mut set_terrain: F, mut set_biome: G)
    where F: FnMut(V2<i32>, TerrainType),
          G: FnMut(V2<i32>, Biome) {
    geomorph::with_cache(|cs| {
        let outside = cs.iter().filter(|c| c.spec.biome == Biome::Overland)
                .collect::<Vec<&Chunk>>();
        let base = cs.iter().filter(|c| c.spec.biome == Biome::Base)
                .collect::<Vec<&Chunk>>();

        for cy in -3i32..4 {
            for cx in -3i32..4 {
                let pos = V2(cx, cy);
                let in_base =
                    Rect(V2(-3, -3), V2(2, 2)).contains(&pos) ||
                    Rect(V2(-3,  2), V2(2, 2)).contains(&pos) ||
                    Rect(V2( 2, -3), V2(2, 2)).contains(&pos) ||
                    Rect(V2( 2,  2), V2(2, 2)).contains(&pos);

                let chunk = rng.choose(
                    if in_base { &base[..] }
                    else { &outside[..] }).unwrap();

                for (&(x, y), &terrain) in chunk.cells.iter() {
                    let pos = herringbone_map((cx, cy), (x, y));
                    set_terrain(pos, terrain);
                    set_biome(pos, if in_base { Biome::Base } else { Biome::Overland });
                }
            }
        }
    });
}


// Map in-chunk coordinates to on-map coordinates based on chunk position in
// the herringbone chunk grid.
fn herringbone_map(chunk_pos: (i32, i32), in_chunk_pos: (i32, i32)) -> V2<i32> {
    let (cx, cy) = chunk_pos;
    let (div, m) = cx.div_mod_floor(&2);
    let (x, y) = in_chunk_pos;

    let origin_x = div * CHUNK_W + cy * CHUNK_W;
    let origin_y = cy * CHUNK_W - m * CHUNK_W - 3 * div * CHUNK_W;

    if m == 0 {
        V2(origin_x + x, origin_y + y)
    } else {
        V2(origin_x + y, origin_y + x)
    }
}

static CHUNK_W: i32 = 11;
