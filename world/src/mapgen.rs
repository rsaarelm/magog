use std::local_data::Ref;
use std::cell::RefCell;
use std::collections::hashmap::{HashMap, HashSet};
use std::rand;
use rand::Rng;
use calx::text::Map2DUtil;
use num::Integer;
use spatial::{Location, DIRECTIONS6, Position};
use system::{World};
use terrain::*;
use system::{};
use spawn::Spawn;
use mobs::{Mobs};
use area::Area;

/// Procedural game world generation.
pub trait MapGen {
    fn gen_herringbone(&mut self, spec: &AreaSpec);
    fn next_level(&mut self);
}

impl MapGen for World {
    // http://nothings.org/gamedev/herringbone/
    fn gen_herringbone(&mut self, spec: &AreaSpec) {
        let mut rng = rand::task_rng();
        let chunkref = get_cache();
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

        for cy in range(-2i, 2) {
            for cx in range(-2i, 2) {
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

    fn next_level(&mut self) {
        self.system_mut().area.clear();
        self.clear_npcs();
        self.system_mut().depth += 1;
        let depth = self.system().depth;

        let spec = AreaSpec::new(
            if depth == 1 { Overland } else { Dungeon },
            depth);

        self.gen_herringbone(&spec);

        let loc = self.spawn_loc().unwrap();
        let mut player = self.player().unwrap();
        player.set_location(loc);
        self.gen_mobs(&spec);

        // FIXME
        //self.system_mut().fx.msg(format!("Level {}", depth).as_slice());
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

local_data_key!(CHUNK_CACHE: RefCell<Vec<Chunk>>)

type Cells = HashMap<(int, int), TerrainType>;

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

struct Chunk {
    cells: Cells,
    connected: bool,
    exit: bool,
    spec: AreaSpec,
}

fn add_cache_chunk(biome: Biome, depth: int, text: &str) {
    assert!(CHUNK_CACHE.get().is_some());

    match Chunk::new(AreaSpec::new(biome, depth), text) {
        Ok(chunk) => CHUNK_CACHE.get().unwrap().borrow_mut().push(chunk),
        Err(e) => fail!("Bad chunk cache data: {}", e),
    }
}

// Only use this to access the cache, make sure the lazy init check gets
// called before access.
fn get_cache() -> Ref<RefCell<Vec<Chunk>>> {
    check_cache();
    CHUNK_CACHE.get().unwrap()
}

fn check_cache() {
    if CHUNK_CACHE.get().is_none() {
        CHUNK_CACHE.replace(Some(RefCell::new(vec!())));
        init_geomorphs();
    }
}

impl Chunk {
    pub fn new(spec: AreaSpec, text: &str) -> Result<Chunk, String> {
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
            spec: spec,
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

// Geomorph data.

/*
1######B222 Template for herringbone prefabs
1##########
1########## Cells at positions A, B and C must have an open tile.
A########## On each half, the openings A, B and C must be connected.
########### The two halves may or may not be connected.
########### This ensures automatic map connectivity, while not
########### making the map trivially open.
##########B
##########2 The numbered lines are parameters by which the openings
##########2 are positioned. When changing the position of an opening
##########2 for an alternative set, lines with the same symbol must
3*********1 remain at equal length.
3*********1
3*********1
3*********A
3**********
C**********
***********
***********
***********
***********
33333C*****
*/

/// Initialize global geomorph cache.
fn init_geomorphs() {
    add_cache_chunk(Overland, 0, "\
%%%%%%%,%%%
%%%%%%%,%%%
%%%%%%%,%%%
,,,%%%,,%%%
%%,,,,,%%%%
%%,%,,,%%%%
%%%,,%,%%%%
%%%%%%%,,,,
%%%%%%%,%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%,%%%
%%%%%%,,,,,
%%%%%,,%%%%
,,,,%,,%%%%
%%%,,,%%%%%
%%%,,%%%%%%
%%%%,%%%%%%
%%%%,%%%%%%
%%%%%,%%%%%");

    add_cache_chunk(Overland, 0, "\
%%%%%%%,%%%
%%%%%%,,%%%
%%%%,,,%%%%
,,,%%,,%%%%
%%,,%%,,%%%
%%%%,,,%%%%
%%%%,,%%%%%
%%%%,,%%,,,
%%%%%,,,,%%
%%%%%,,%%%%
%%%,,,%%%%%
%%%%,,%%%%%
%%%%,,%%%%%
%%%%,%%%%%%
%%%%,,%,,,,
%%%%%%,,%%%
,,%%%,,%%%%
%%,,,,%%%%%
%%%%,,%%%%%
%%%%%,,%%%%
%%%%%,,%%%%
%%%%%,%%%%%");

    add_cache_chunk(Overland, 0, "\
%%%%%,,,%%%
%%,,,,,,,%%
%,,,%,,,,,%
,,,,,,%,,%%
,,,,,,,,,,%
%,,,,,,,,,%
%,,,%,,%,,,
%%o,,,,,,,,
%%,,,,,%,,,
%,,,,%,,,,%
%%,,,,,,,%%
%%,%,,,o,,%
%%,,,,,,,%%
%%%,,,,o%%%
%%%%%,,,,,,
%%%%%%%%%,%
,,,,%%%%,,%
%%%,%%%%,%%
%%%%,%%,,%%
%%%%%,,,%%%
%%%%%,%%%%%
%%%%%,%%%%%");

    add_cache_chunk(Overland, 0, "\
%%%%%,,,%%%
%%%,,,,,,,%
%%,,,,,,,,%
,,,,%,%,,,%
,,,,,,,,,,%
,,,%,,,,,,%
%,,,o,,,,,,
%,,,,%,,,,,
%,%,,,,%,,%
%,,,,,o,,,%
%%,%,,,,%%%
%,,,,,,,o%%
%%o,,,,,,,%
%%,,,,,,,,,
%,,,%,,,,,,
%,,,,,,,,,%
,,,%,,,%,,%
,,,,,,,,,%%
%,,,,,,,,,%
%%o%,,,,,%%
%%%,,,,,%%%
%%%%,,,%%%%");

    add_cache_chunk(Overland, 0, "\
%%%oo,,,%%%
%,,o=o,,%%%
%%,,=%,,,%%
,,,,=%,,,%%
%,%,==,,,%%
%,,,,=,%,%%
%,,%,=,,,,%
%,,,==,,,,,
,,,,=o,,,%%
,,%,=,,%,%%
%,,,==,,,%%
%,,,,=,,%%%
,,%%,=,,,%%
,,%,,=o,,%%
%,,,==,,,,,
,,,~===~~,,
,,~=====~~,
,,~====~~,,
,,~=====~,,
,,~~===~~,,
,,,~~~~~~,,
%,,,,,,,,%%");

    add_cache_chunk(Overland, 0, "\
%%%%%%%,%%%
%%%,,,,,%%%
%%,,%%%,%%%
,,,%%%%,,%%
%%,,%%,,,%%
%%%,,,,%%%%
%%,,,,,,,,%
%%,,,,,%%%,
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%,,,%%%
%%%,,,,,%%%
%%%,,,,,,,%
%%,,..,,,,%
%%,A./..,,,
%,,..//.,,%
,,,.//.A,%%
%%,,./.,,%%
%,,,A,.,,%%
%%,,,,,,,%%
%%%,,,,,%%%
%%%%%,%%%%%");

    add_cache_chunk(Overland, 0, "\
%%%%%%%,%%%
%%%%%,,,%%%
%%,,,,,,%%%
,,,,,,,%,,%
%%,,,,,%%,%
%/%,,,,,%%%
%%%%/%%,/%%
%%%%%%%,,,,
%%%%%%%,/%%
%%%%%,,,%%%
%///..//%%%
%/.A..../%%
%.A..A..,%%
%./.a../%,%
%%.A..A/%,,
%%...A,,%%%
,,,,,//%/%%
%%%,,%%%%%%
%%%%,%%/%%%
%%%%%,%%%%%
%%%%%,,%%%%
%%%%%,,%%%%");

    add_cache_chunk(Overland, 0, "\
%%%%%%%,%%%
%%%%%%,,,%%
%%%%,,,,,%%
,,,,,,,,,%%
%%,,%%,,,%%
%%%%%,,,%%%
%%%%%%,,%,%
%%%%,,,,,,,
%%%%,%%,,,%
%%%%%%%,,%%
%%%%%%%,,%%
%%%%%,,,%%%
%%%%%,%%%%%
%%%%%,%%%%%
%%%%%,,,,,,
%%,,,,,,,,%
,,,,,,,%%%%
%%%,,,%%%%%
%%%%,,,%%%%
%%%%%,,,%%%
%%%%%,,%%%%
%%%%%,,%%%%");

    add_cache_chunk(Overland, 0, "\
%%%%%%,,%%%
%%%%%%,,,%%
%%%%,,,,,%%
,,,,,,,,,%%
%%,,%%,,,%%
%%%%%,,,%%%
%%%%%%,,%,%
%%%o,,,,,,,
%%%~~%%,,,%
==%~~~o,,~%
=======~~~=
===========
~====~~~===
,,,~~~%%%%%
,%%%%,,,,,,
%%,,,,,,,,%
,,,,,,,%%%%
%%%,,,%%%%%
%%%%,,,%%%%
%%%%%,,,%%%
%%%%%,,%%%%
%%%%%,,%%%%");

    add_cache_chunk(Overland, 0, "\
,,,,,,,,,,,
,========,,
,=#|#=#|#=,
,=|.###.|=,
,=#.+.+.#=,
,=###+###=,
,==#b.b#==,
,==#>.b#==,
,==|...|==,
,==##+##==,
,..+...+..,
,..+...+..,
,==##+##==,
,==|...|==,
,==#.T.#==,
,==#...#==,
,=###+###=,
,=#.+.+.#=,
,=|.###.|=,
,=#|#=#|#=,
,,========,
,,,,,,,,,,,");

    add_cache_chunk(Overland, 0, "\
,,,,,,,,,,,
,,,,,,,,,,,
,,******,,,
,****..**,,
,*.*.....,,
,*.**!..*,,
,*.**..**,,
,*....***,,
,**I.III**,
,,**..****,
,**....!**,
,**.*.***,,
,***.**X*,,
,*...XXX*,,
,**...X**,,
,*##+#XX*,,
,*#....X**,
,*#..>X#**,
,**!..X#**,
,,**###***,
,,*******,,
,,,,,,,,,,,");

    add_cache_chunk(Dungeon, 0, "\
#######.###
#######+###
##......###
.+......###
##......###
##g.....###
##......###
##g.....+..
##......###
###+#######
###.#######
###.#######
###.#######
###.#######
###........
#####.#####
......#####
#####.#####
#####.#####
#####.#####
#####.#####
#####.#####");

    add_cache_chunk(Dungeon, 0, "\
#######.###
#######.###
#######.###
........###
#######.###
#######.###
#######.###
#######....
###########
###########
###########
###########
###########
###########
#####......
#####.#####
......#####
#####.#####
#####.#####
#####.#####
#####.#####
#####.#####");

    add_cache_chunk(Dungeon, 0, "\
#######.###
#######+###
##.......##
.+...#...##
##..###..##
##...#...##
##.#...#.##
##.......+.
##.......##
##.#...#.##
##.......##
##.......##
##.#...#.##
##.......##
##.......+.
##.#...#.##
.+...#...##
##..###..##
##...#...##
##.......##
#####+#####
#####.#####");

    add_cache_chunk(Dungeon, 0, "\
*******.***
****#...***
#####...#**
.......##**
##.....+..*
*..##..#..*
**..#++##**
**#|#......
**....!..**
**..!.....*
**..XX..!.*
**!.XXX..**
**...XX..**
**X......**
**XXXXX....
**.XXX...**
.......X.**
**..!..XX**
***....XX**
****..**XX*
*****.*XXX*
*****.*****");

    add_cache_chunk(Dungeon, 0, "\
#######.###
#######.###
#######.###
........###
#.......###
#.......###
#.......###
#..........
###=====###
###=====###
###=====###
###=====###
###IIIII###
###.....###
###........
###.....###
........###
###.....###
###.....###
###.....###
#####.#####
#####.#####");

    add_cache_chunk(Dungeon, 0, "\
#######.###
#.........#
#.........#
..........#
#.........#
#.........#
#.........#
#..........
#..#.#....#
#.........#
#..#.>.#..#
#.........#
#....#.#..#
#.........#
#..........
#.........#
..........#
#.........#
#.........#
#.........#
#.........#
#####.#####");
}
