use std::iter;
use num::{Integer};
use rand::{Rng};
use calx::{V2};
use geomorph;
use geomorph::{Chunk};
use ::{StaticArea, FormType, AreaSpec};

static CHUNK_W: i32 = 11;

/// Generate a map using Herringbone Wang Tiles.
///
/// Designed after http://nothings.org/gamedev/herringbone/
pub fn herringbone<R: Rng>(
    rng: &mut R, spec: &AreaSpec) -> StaticArea<FormType> {

    // Generate the terrain.
    let mut area = geomorph::with_cache(|cs| generate_terrain(rng, spec, cs));

    // Place the spawns.

    // No connectivity analysis yet, trusting that herringbone map has
    // total connectivity. Later on, use Dijkstra map that spreads from
    // entrance/exit as a reachability floodfill to do something cleverer
    // here.
    let mut opens: Vec<V2<i32>> = area.terrain.iter()
        .filter(|&(_, &t)| t.valid_spawn_spot())
        .map(|(&loc, _)| loc)
        .collect();

    rng.shuffle(&mut opens);

    area.player_entrance = opens.pop().unwrap();

    let num_mobs = 32;
    let num_items = 12;

    area.spawns.extend(
        iter::repeat(FormType::Creature).take(num_mobs).chain(
        iter::repeat(FormType::Item).take(num_items))
        .filter_map(|spawn| {
            if let Some(loc) = opens.pop() {
                Some((loc, spawn))
            } else {
                None
            }
        })
    );

    area
}

fn generate_terrain<R: Rng>(rng: &mut R, spec: &AreaSpec, cs: &Vec<Chunk>) -> StaticArea<FormType> {
    let mut area = StaticArea::new();

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
                area.terrain.insert(herringbone_pos((cx, cy), (x, y)), terrain);
            }
        }
    }

    area
}

// Map in-chunk coordinates to on-map coordinates based on chunk position in
// the herringbone chunk grid.
fn herringbone_pos(chunk_pos: (i32, i32), in_chunk_pos: (i32, i32)) -> V2<i32> {
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
