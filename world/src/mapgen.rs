use num::{Integer};
use rand::Rng;
use util::{V2};
use util::text::Map2DUtil;
use geomorph;
use geomorph::{Chunk};
use terrain::TerrainType;
use terrain::TerrainType::*;

pub fn gen_herringbone<R: Rng, F>(
    rng: &mut R, spec: &::AreaSpec, mut set_terrain: F)
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
                    if (cx, cy) == (exit_x, exit_y) { exit.as_slice() }
                    else if on_edge { edge.as_slice() }
                    else { inner.as_slice() }).unwrap();

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

pub fn gen_prefab<F>(mut set_terrain: F)
    where F: FnMut(i32, i32, i32, TerrainType) {

    static FLOORS: [&'static str; 7] = [

"\
________________
________________
________________
________________
____#########___
____#####___#___
____#########___
____#########___
____#########___
____#########___
____#########___
____#########___
____#########___
____#########___
________________
________________",

"\
________________
________________
________________
________________
____#########___
____#___##__#___
____|_______#___
____#_______|___
____#_______#___
____#_______#___
____#_______#___
____#_______#___
____#_______#___
____##|#_####___
________________
________________",

"\
________________
________________
________________
________________
____#########___
____#____##_#___
____#_______#___
____#_______#___
____#_______#___
____#_______#___
____#_______#___
____#_______#___
____#_______#___
____####_####___
________________
________________",

"\
_***____________
*_**____________
**_*____________
***____####_____
____#########___
____#########___
____#######_#___
____#######_#___
____#########___
____#########___
____#########___
____#########___
____#########___
____#########___
_______**_______
________________",

"\
_****___________
*_**______%_____
**_*____________
***____#__#___#_
____####__###___
____#______##___
____|______##___
____#_______|___
____________#___
____#_______#___
____####____|___
____####____#___
____####____#___
____#########___
*______******___
**______________",

"\
_*****__________
*_***_____#__*__
**_*____________
***____#__#___*_
_%__####__###___
%___#______##___
____#______##___
%___#______##___
____________#___
____#_______#___
____####____#___
____####____#___
____####____#___
**__#########___
***____******___
****_____**_____",

"\
*************___
**************__
***************_
***************_
****#########***
****#########*_*
****#########***
****#########***
****#########***
****#########***
****#########***
****#########***
****#########***
****#########***
****************
****************",
    ];

    for z in 0..(FLOORS.len()) {
        for (glyph, x, y) in FLOORS[z].chars().map2d() {
            let terrain = legend(glyph).expect("Unknown glyph");
            set_terrain(x, y, (z as i32), terrain);
        }
    }

    fn legend(glyph: char) -> Option<TerrainType> {
        match glyph {
            '.' => Some(Floor),
            '#' => Some(Wall),
            '~' => Some(Shallows),
            '=' => Some(Water),
            ',' => Some(Grass),
            '+' => Some(Door),
            '*' => Some(Rock),
            'X' => Some(Magma),
            '|' => Some(Window),
            '%' => Some(Tree),
            '/' => Some(DeadTree),
            'x' => Some(Battlement),
            'o' => Some(Stone),
            'A' => Some(Menhir),
            'g' => Some(Grave),
            'b' => Some(Barrel),
            'T' => Some(Table),
            'a' => Some(Altar),
            'I' => Some(Bars),
            '!' => Some(Stalagmite),
            ';' => Some(TallGrass),
            '>' => Some(Downstairs),
            '_' => Some(Space),
            _ => None
        }
    }
}
