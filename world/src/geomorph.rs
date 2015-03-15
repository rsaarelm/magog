use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use util::text::Map2DUtil;
use terrain::TerrainType::*;
use terrain::TerrainType;
use {AreaSpec, Biome};
use geomorph_data;
use dir6::Dir6;

thread_local!(static CHUNK_CACHE: RefCell<Vec<Chunk>> = RefCell::new(vec![]));

type Cells = HashMap<(i32, i32), TerrainType>;

pub struct Chunk {
    pub cells: Cells,
    pub connected: bool,
    pub exit: bool,
    pub spec: AreaSpec,
}

pub fn add_cache_chunk(biome: Biome, depth: i32, text: &str) {
    CHUNK_CACHE.with(|c| {
        match Chunk::new(AreaSpec::new(biome, depth), text) {
            Ok(chunk) => c.borrow_mut().push(chunk),
            Err(e) => panic!("Bad chunk cache data: {}", e),
        }
    });
}

// Only use this to access the cache, make sure the lazy init check gets
// called before access.
pub fn with_cache<A, F>(mut f: F) -> A
    where F: FnMut(&Vec<Chunk>) -> A {
    check_cache();
    CHUNK_CACHE.with(|c| f(&*c.borrow()))
}

fn check_cache() {
    if CHUNK_CACHE.with(|c| c.borrow().len() == 0) {
        geomorph_data::init_geomorphs();
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
        'x' => Some(Wall),
        'o' => Some(Stone),
        'b' => Some(Barrel),
        'T' => Some(Table),
        'I' => Some(Wall),
        '!' => Some(Stalagmite),
        ';' => Some(TallGrass),
        'q' => Some(Void),
        'c' => Some(Crater),
        _ => None
    }
}

impl Chunk {
    pub fn new(spec: AreaSpec, text: &str) -> Result<Chunk, String> {
        let chunk_w = 11;
        let chunk_h = 22;

        let mut cells: Cells = HashMap::new();
        for (glyph, x, y) in text.chars().map2d() {
            if x >= chunk_w || y >= chunk_h {
                println!("{}", text);
                return Err("Bad chunk size.".to_string());
            }

            // Don't insert anything. Use around big crater to not overwrite
            // the edges.
            if glyph == ' ' {
                continue;
            }

            // Special case, big crater
            if glyph == 'C' {
                cells.insert((x, y), Floor);
                cells.insert((x - 1, y - 1), CraterN);
                cells.insert((x    , y - 1), CraterNE);
                cells.insert((x + 1, y    ), CraterSE);
                cells.insert((x + 1, y + 1), CraterS);
                cells.insert((x    , y + 1), CraterSW);
                cells.insert((x - 1, y    ), CraterNW);

                continue;
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
            spec: spec,
        })
    }
}

fn verify_topology(regions: &Vec<HashSet<(i32, i32)>>) -> Option<String> {
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
        r1 = &regions[0];
        r2 = &regions[0];
    } else {
        assert!(regions.len() == 2);
        if regions[0].contains(&set_1[0]) {
            r1 = &regions[0];
            r2 = &regions[1];
        } else {
            r1 = &regions[1];
            r2 = &regions[0];
        }
    }

    for p in set_1.iter() {
        if !r1.contains(p) {
            return Some(format!("Top region missing connection to cell {:?}", p));
        }
    }

    for p in set_2.iter() {
        if !r2.contains(p) {
            return Some(format!("Bottom region missing connection to cell {:?}", p));
        }
    }

    None
}

fn make_topology(cells: &Cells) -> Vec<HashSet<(i32, i32)>> {
    let mut open : HashSet<(i32, i32)> = cells.iter()
        .filter(|&(_p, t)| !t.blocks_walk())
        .map(|(&p, _t)| p)
        .collect();
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

/// Split a point set into an arbitrary connected region and the remaining set.
fn split_connected(set: &HashSet<(i32, i32)>) ->
(HashSet<(i32, i32)>, HashSet<(i32, i32)>) {
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

                for dir in Dir6::iter() {
                    let d = dir.to_v2();
                    let edge_point = (x + d.0, y + d.1);
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
