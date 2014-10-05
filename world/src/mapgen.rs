use std::rand;
use rand::Rng;
use num::Integer;
use calx::{V2};
use location::{Chart};
use geomorph;
use geomorph::{Chunk};

pub fn gen_herringbone<C: Chart>(origin: &C, spec: &AreaSpec) {
    // TODO: Determine level rng from world seed.
    let mut rng = rand::task_rng();
    let chunkref = geomorph::get_cache();
    let chunkbor = chunkref.borrow();
    let chunks = chunkbor.iter().filter(
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

    let exit_x = rng.gen_range(-1, 0);
    let exit_y = rng.gen_range(-1, 0);

    for cy in range(-2i, 2) {
        for cx in range(-2i, 2) {
            let on_edge = cy == -2 || cx == -2 || cy == 1 || cx == 1;

            let chunk = rng.choose(
                if (cx, cy) == (exit_x, exit_y) { exit.as_slice() }
                else if on_edge { edge.as_slice() }
                else { inner.as_slice() }).unwrap();

            for (&(x, y), &terrain) in chunk.cells.iter() {
                let loc = *origin + herringbone_map((cx, cy), (x, y));
                loc.set_terrain(Some(terrain));
            }
        }
    }
}

#[deriving(PartialEq)]
pub enum Biome {
    Overland = 0b1,
    Dungeon  = 0b10,

    // For things showing up at a biome.
    Anywhere = 0b11111111,
}

pub struct AreaSpec {
    pub biome: Biome,
    pub depth: int,
}

impl AreaSpec {
    pub fn new(biome: Biome, depth: int) -> AreaSpec {
        AreaSpec { biome: biome, depth: depth }
    }

    pub fn can_spawn(&self, environment: &AreaSpec) -> bool {
        self.depth >= 0 && self.depth <= environment.depth &&
        (self.biome as int & environment.biome as int) != 0
    }
}

// Map in-chunk coordinates to on-map coordinates based on chunk position in
// the herringbone chunk grid.
fn herringbone_map(chunk_pos: (int, int), in_chunk_pos: (int, int)) -> V2<int> {
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

static CHUNK_W: int = 11;
