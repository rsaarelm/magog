use collections::hashmap::{HashMap, HashSet};
use std::rand;
use rand::Rng;
use text::Map2DUtil;
use num::Integer;
use world::world::{World, Location};
use world::terrain::*;
use world::world::{DIRECTIONS6};
use world::spawn::Spawn;
use world::mobs::{Mobs};
use world::geomorph::Chunks;

pub trait MapGen {
    fn gen_herringbone(&mut self, chunks: &Vec<Chunk>);
    fn next_level(&mut self, chunks: &Chunks);
}

impl MapGen for World {
    // http://nothings.org/gamedev/herringbone/
    fn gen_herringbone(&mut self, chunks: &Vec<Chunk>) {
        let mut rng = rand::task_rng();

        let edge: Vec<&Chunk> = chunks.iter().filter(|c| !c.exit && c.connected).collect();
        let inner: Vec<&Chunk> = chunks.iter().filter(|c| !c.exit).collect();
        let exit: Vec<&Chunk> = chunks.iter().filter(|c| c.exit).collect();

        assert!(!exit.is_empty(), "No exit chunks found");
        assert!(inner.len() + exit.len() == chunks.len());

        for cy in range(-2, 2) {
            for cx in range(-2, 2) {
                let on_edge = cy == -2 || cx == -2 || cy == 1 || cx == 1;

                let chunk = rng.choose(
                    if (cx, cy) == (0, 0) { exit.as_slice() }
                    else if on_edge { edge.as_slice() }
                    else { inner.as_slice() }).unwrap();

                for (&(x, y), &terrain) in chunk.cells.iter() {
                    let (ax, ay) = herringbone_map((cx, cy), (x, y));
                    self.terrain_set(Location::new(ax as i8, ay as i8), terrain);
                }
            }
        }
    }

    fn next_level(&mut self, chunks: &Chunks) {
        // TODO: Preserve player object.
        self.area.clear();
        self.clear_npcs();
        self.depth += 1;

        self.gen_herringbone(
            if self.depth == 1 { &chunks.overland }
            else { &chunks.dungeon });

        let loc = self.spawn_loc().unwrap();
        let player = self.player().unwrap();
        self.mut_mob(player).loc = loc;
        self.gen_mobs();
    }
}

// Map in-chunk coordinates to on-map coordinates based on chunk position in
// the herringbone chunk grid.
fn herringbone_map(chunk_pos: (int, int), in_chunk_pos: (int, int)) -> (int, int) {
    let (cx, cy) = chunk_pos;
    let (div, m) = cx.div_mod_floor(&2);
    let (x, y) = in_chunk_pos;

    let origin_x = div * CHUNK_W + cy * CHUNK_W;
    let origin_y = cy * CHUNK_W - m * CHUNK_W - 3 * div * CHUNK_W;

    if m == 0 {
        (origin_x + x, origin_y + y)
    } else {
        (origin_x + y, origin_y + x)
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
        'x' => Some(Fence),
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
        _ => None
    }
}

static CHUNK_W: int = 11;

type Cells = HashMap<(int, int), TerrainType>;

pub struct Chunk {
    cells: Cells,
    connected: bool,
    exit: bool,
}

impl Chunk {
    pub fn new(text: &str) -> Result<Chunk, String> {
        let chunk_w = 11;
        let chunk_h = 22;

        let mut cells = HashMap::new();
        for (glyph, x, y) in text.chars().map2d() {
            if x >= chunk_w || y >= chunk_h {
                println!("{}", text);
                return Err("Bad chunk size.".to_string());
            }
            cells.insert((x, y), match legend(glyph) {
                Some(t) => t,
                None => {
                    println!("{}", text);
                    return Err(format!("Unrecognized chunk glyph {}", glyph));
                }
            });
        }

        let regions = make_topology(&cells);
        match verify_topology(&regions) {
            Some(err) => {
                println!("{}", text);
                return Err(err);
            }
            None => (),
        }

        let exit = cells.iter().any(|(_p, t)| t.is_exit());

        Ok(Chunk {
            cells: cells,
            connected: regions.len() == 1,
            exit: exit,
        })
    }
}

fn verify_topology(regions: &Vec<HashSet<(int, int)>>) -> Option<String> {
    let chunk_dim = 11;
    let span_1 = 3;
    let span_2 = 7;
    let span_3 = 5;

    let set_1 = vec!((0, span_1), (span_2, 0), (chunk_dim - 1, span_2));
    let set_2 = vec!(
        (0, chunk_dim + span_3),
        (span_3, chunk_dim * 2 - 1),
        (chunk_dim - 1, chunk_dim + span_1));

    if regions.len() < 1 || regions.len() > 2 {
        return Some(format!("Bad number of connected regions {}", regions.len()));
    }

    let mut r1;
    let mut r2;

    if regions.len() == 1 {
        r1 = regions.get(0);
        r2 = regions.get(0);
    } else {
        assert!(regions.len() == 2);
        if regions.get(0).contains(set_1.get(0)) {
            r1 = regions.get(0);
            r2 = regions.get(1);
        } else {
            r1 = regions.get(1);
            r2 = regions.get(0);
        }
    }

    for p in set_1.iter() {
        if !r1.contains(p) {
            return Some(format!("Top region missing connection to cell {}", p));
        }
    }

    for p in set_2.iter() {
        if !r2.contains(p) {
            return Some(format!("Bottom region missing connection to cell {}", p));
        }
    }

    None
}

fn make_topology(cells: &Cells) -> Vec<HashSet<(int, int)>> {
    let mut open = open_cells(cells);
    let mut ret = vec!();

    if cells.is_empty() {
        return ret;
    }

    loop {
        let (connected, rest) = split_connected(&open);
        assert!(!connected.is_empty());
        ret.push(connected);
        if rest.is_empty() {
            return ret;
        }
        assert!(open.len() > rest.len());
        open = rest;
    }
}

fn open_cells(cells: &Cells) -> HashSet<(int, int)> {
    cells.iter()
        .filter(|&(_p, t)| t.is_walkable())
        .map(|(&p, _t)| p)
        .collect()
}

/// Split a point set into an arbitrary connected region and the remaining set.
fn split_connected(set: &HashSet<(int, int)>) ->
(HashSet<(int, int)>, HashSet<(int, int)>) {
    if set.is_empty() {
        return (HashSet::new(), HashSet::new());
    }

    let mut connected = HashSet::new();
    let mut edge = vec!();
    let mut rest = set.clone();

    let first = set.iter().next().unwrap();
    rest.remove(first);
    edge.push(*first);

    loop {
        match edge.pop() {
            Some(point) => {
                assert!(!connected.contains(&point));
                assert!(!rest.contains(&point));
                connected.insert(point);

                let (x, y) = point;

                for d in DIRECTIONS6.iter() {
                    let edge_point = (x + d.x, y + d.y);
                    if rest.contains(&edge_point) {
                        assert!(!connected.contains(&edge_point));
                        rest.remove(&edge_point);
                        edge.push(edge_point);
                    }
                }
            }
            None => { break; }
        }
    }

    (connected, rest)
}
