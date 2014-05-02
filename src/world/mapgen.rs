use std::cmp::max;
use rand::Rng;
use collections::hashmap::HashSet;
use num::Integer;

use text::Map2DUtil;
use cgmath::aabb::{Aabb, Aabb2};
use cgmath::point::{Point2};
use world::area::{Area, DIRECTIONS6, Location};
use world::area;

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

static CHUNK_W: int = 11;
/*
"\
....... ...
...........
...........
 ..........
...........
...........
...........
..........
...........
...........
...........
...........
...........
...........
..........
...........
 ..........
...........
...........
...........
...........
..... .....",
*/

static CHUNKS: &'static[&'static str] = &[
    /*
"\
%%%%%%%,%%%
%%%%%%%%%%%
%%%%%%%%%%%
,%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%,
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%,
%%%%%%%%%%%
,%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%%%%%%%
%%%%%,%%%%%",
*/

"\
,,,,,,,,,,,
,######|##,
,#..#....#,
,|..+..T.|,
,#..#....#,
,##+######,
,#....#,,x,
,|.T..+,,,,
,#....#,%x,
,###|##,,x,
,,,,,xb,,x,
,,,%,xx,xx%
,,%,,,,,%%%
,,,,,,,,,%%
,,,%,;,,,,%
,,,,;;;,,,,
%%,;;~~,,,,
%%,;~~~~~,,
%,,;~~~~~,,
,,,,~~~,,,%
,,,,,,,,%%%
,,,,,,,%%%%",

"\
%%%%%%%,%%%
%%%%%,,,%%%
%%,,,,,,%%%
,,,,,,,%,,%
%%,,,,,%%,%
%/%,,,,,%%%
%%%%/%%,/%%
%%%%,%%,,,,
%%%%%%%%/%%
%%%%%%%%%%%
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
%%%%%,,%%%%",

"\
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
%%%%%,,%%%%",

"\
%%%%%%,,%%%
%%%%%%,,,%%
%%%%,,,,,%%
,,,,,,,,,%%
%%,,%%,,,%%
%%%%%,,,%%%
%%%%%%,,%,%
%%%o,,,,,,,
%%%~~%%,,,%
,~%~~~o,,~%
~~=====~~~=
~==========
~====~~~~,%
,,%~~~%%%%%
,%%%%,,,,,,
%%,,,,,,,,%
,,,,,,,%%%%
%%%,,,%%%%%
%%%%,,,%%%%
%%%%%,,,%%%
%%%%%,,%%%%
%%%%%,,%%%%",

"\
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
#####.#####",

"\
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
#####.#####",

"\
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
#####.#####",

"\
*******.***
****#...***
#####...#**
.......##**
##.....#..*
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
*****.*****",

"\
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
#####.#####",
];

pub trait MapGen {
    fn gen_cave<R: Rng>(&mut self, rng: &mut R, make_exit: bool);
    fn gen_prefab(&mut self, prefab: &str);
    fn gen_herringbone<R: Rng>(&mut self, rng: &mut R);
}

impl MapGen for Area {
    fn gen_cave<R: Rng>(&mut self, rng: &mut R, make_exit: bool) {
        let center = Location::new(0i8, 0i8);
        let mut edge = HashSet::new();
        let bounds = Aabb2::new(Point2::new(-16i8, -16i8), Point2::new(16i8, 16i8));
        let mut dug = 1;
        self.dig(center);
        for &v in DIRECTIONS6.iter() {
            edge.insert(center + v);
        }

        for _itercount in range(0, 10000) {
            let loc = **rng.sample(edge.iter(), 1).get(0);
            let nfloor = DIRECTIONS6.iter().count(|&v| self.is_open(loc + v));
            assert!(nfloor > 0);

            // Weight digging towards narrow corners.
            if rng.gen_range(0, max(1, nfloor)) != 0 {
                continue;
            }

            self.dig(loc);
            edge.remove(&loc);
            dug += 1;

            for &v in DIRECTIONS6.iter() {
                let p = loc + v;
                if self.get(p) == self.default && bounds.contains(p.p()) {
                    edge.insert(p);
                }
            }

            if dug > 384 { break; }
        }

        if make_exit {
            let down_pos = **rng.sample(edge.iter(), 1).get(0);
            self.set(down_pos, area::Downstairs);
            edge.remove(&down_pos);
        }

        // Depillar
        for &loc in edge.iter() {
            let nfloor = DIRECTIONS6.iter().count(|&v| self.is_open(loc + v));
            assert!(nfloor > 0);
            if nfloor == 6 {
                self.set(loc, area::Stalagmite);
            }
        }
    }

    fn gen_prefab(&mut self, prefab: &str) {
        for (c, x, y) in prefab.chars().map2d() {
            if c == '.' {
                self.set.insert(Location::new(x as i8, y as i8), area::Floor);
            }
            if c == '~' {
                self.set.insert(Location::new(x as i8, y as i8), area::Water);
            }
        }

    }

    // http://nothings.org/gamedev/herringbone/
    fn gen_herringbone<R: Rng>(&mut self, rng: &mut R) {
        for cy in range(-3, 4) {
            for cx in range(-3, 4) {
                let chunk = rng.choose(CHUNKS);
                for (glyph, x, y) in chunk.chars().map2d() {
                    let terrain = match glyph {
                        '.' => area::Floor,
                        '#' => area::Wall,
                        '~' => area::Shallows,
                        '=' => area::Water,
                        ',' => area::Grass,
                        '+' => area::Door,
                        '*' => area::Rock,
                        'X' => area::Magma,
                        '|' => area::Window,
                        '%' => area::Tree,
                        '/' => area::DeadTree,
                        'x' => area::Fence,
                        'o' => area::Stone,
                        'A' => area::Menhir,
                        'g' => area::Grave,
                        'b' => area::Barrel,
                        'T' => area::Table,
                        'a' => area::Altar,
                        'I' => area::Bars,
                        '!' => area::Stalagmite,
                        ';' => area::TallGrass,
                        _ => area::Void
                    };
                    let (ax, ay) = herringbone_map((cx, cy), (x, y));
                    self.set.insert(Location::new(ax as i8, ay as i8), terrain);
                }
            }
        }
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
