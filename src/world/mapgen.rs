use num::{Integer};
use rand::Rng;
use calx::{V2};
use super::geomorph;
use super::geomorph::{Chunk};
use super::terrain::TerrainType;
use super::{AreaSpec};

pub fn gen_herringbone<R: Rng, F>(
    rng: &mut R, spec: &AreaSpec, mut set_terrain: F)
    where F: FnMut(V2<i32>, TerrainType) {
    geomorph::with_cache(|cs| {
        let chunks = cs.iter().filter(
                |c| c.spec.biome == spec.biome && c.spec.depth <= spec.depth)
                .collect::<Vec<&Chunk>>();

        let edge = chunks.iter().filter(|c| !c.exit && c.connected)
            .map(|&c| c).collect::<Vec<&Chunk>>();
        let inner = chunks.iter().filter(|c| !c.exit)
            .map(|&c| c).collect::<Vec<&Chunk>>();
        let exit = chunks.iter().filter(|c| c.exit)
            .map(|&c| c).collect::<Vec<&Chunk>>();

        assert!(!exit.is_empty(), "No exit chunks found");
        assert!(inner.len() + exit.len() == chunks.len());

        let exit_x = rng.gen_range(-2, 2);
        let exit_y = rng.gen_range(-2, 2);

        for cy in -2i32..2 {
            for cx in -2i32..2 {
                let on_edge = cy == -2 || cx == -2 || cy == 1 || cx == 1;

                let chunk = rng.choose(
                    if (cx, cy) == (exit_x, exit_y) { &exit[..] }
                    else if on_edge { &edge[..] }
                    else { &inner[..] }).unwrap();

                for (&(x, y), &terrain) in chunk.cells.iter() {
                    set_terrain(herringbone_map((cx, cy), (x, y)), terrain);
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
