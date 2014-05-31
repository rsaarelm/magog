use std::num::Bounded;
use std::cmp::{min, max};
use std::str;
use std::str::StrAllocating;
use std::vec::Vec;
use collections::hashmap::{HashMap};
use cgmath::point::{Point2};

#[deriving(Eq, TotalEq, Hash, Clone, Decodable, Encodable)]
pub struct Cell {
    pub terrain_type: String,
    pub spawns: Vec<String>,
}

impl Cell {
    pub fn new(terrain_type: String, spawns: Vec<String>) -> Cell {
        Cell { terrain_type: terrain_type, spawns: spawns }
    }
}

#[deriving(Eq, TotalEq, Hash, Clone, Decodable, Encodable)]
pub struct LegendEntry {
    pub glyph: char,
    pub terrain_type: String,
    pub spawns: Vec<String>,
}

// This struct is designed to be serialized with a structured markup language,
// therefore the very simple structure like spelling out the offset coordinates
// instead of using a Point2 type.
#[deriving(Decodable, Encodable)]
pub struct AsciiMap {
    pub offset_x: int,
    pub offset_y: int,
    pub default_terrain: String,
    pub terrain: Vec<String>,
    pub legend: Vec<LegendEntry>,
}

impl AsciiMap {
    pub fn new<I: Iterator<(Point2<int>, Cell)>>(default_terrain: String, mut cells: I) -> AsciiMap {
        let mut terrainMap = HashMap::new();
        let mut legend = HashMap::new();
        let mut lb = LegendBuilder::new();

        let mut min_x = Bounded::max_value();
        let mut min_y = Bounded::max_value();
        let mut max_x = Bounded::min_value();
        let mut max_y = Bounded::min_value();
        for (p, c) in cells {
            min_x = min(min_x, p.x);
            min_y = min(min_y, p.y);
            max_x = max(max_x, p.x + 1);
            max_y = max(max_y, p.y + 1);

            let glyph = lb.glyph(&c);
            terrainMap.insert((p.x, p.y), glyph);
            legend.insert(glyph, c);
        }

        let mut terrain = vec!();
        for _ in range(min_y, max_y) {
            terrain.push(str::from_chars(Vec::from_elem((max_x - min_x) as uint, ' ').as_slice()));
        }

        for (&(x, y), glyph) in terrainMap.iter() {
            // XXX: Can't have unicode.
            unsafe {
                terrain.get_mut((y - min_y) as uint).as_mut_bytes()[(x - min_x) as uint] =
                    *glyph as u8;
            }
        }

        AsciiMap {
            offset_x: min_x,
            offset_y: min_y,
            default_terrain: default_terrain,
            terrain: terrain.iter().map(|x| x.to_string()).collect(),
            legend: legend.iter().map(|(&glyph, cell)| LegendEntry {
                glyph: glyph, terrain_type: cell.terrain_type.to_string(), spawns: cell.spawns.clone()
            }).collect(),
        }
    }
}

struct LegendBuilder {
    lookup: HashMap<Cell, char>,
    floors: Vec<char>,
    walls: Vec<char>,
    others: Vec<char>,
    spawns: Vec<char>,
}

impl LegendBuilder {
    fn new() -> LegendBuilder {
        // The last character in the strings gets used first.
        LegendBuilder {
            lookup: HashMap::new(),
            floors: ":;`'_,.".chars().collect(),
            walls: "$0*#".chars().collect(),
            others: "}{><?)(!^@=&%+".chars().collect(),
            spawns: "987654321ZYXWVUTSRQPONMLKJIHGFEDCBAzyxwvutsrqponmlkjihgfedcba".chars().collect(),
        }
    }

    fn glyph(&mut self, cell: &Cell) -> char {
        match self.lookup.find(cell) {
            Some(&ret) => { return ret; }
            _ => ()
        };
        let mut pick_set =
        match classify(cell) {
            Floor => vec!(&mut self.floors, &mut self.others, &mut self.walls, &mut self.spawns),
            Wall => vec!(&mut self.walls, &mut self.others, &mut self.floors, &mut self.spawns),
            Other => vec!(&mut self.others, &mut self.walls, &mut self.spawns, &mut self.floors),
            Spawning => vec!(&mut self.spawns, &mut self.others, &mut self.walls, &mut self.floors),
        };
        for i in pick_set.mut_iter() {
            match i.pop() {
                Some(ret) => {
                    self.lookup.insert(cell.clone(), ret);
                    return ret;
                }
                _ => (),
            };
        }
        fail!("Ran out of legend glyphs");
    }
}

enum TerrainClass {
    Floor,
    Wall,
    Other,
    Spawning,
}

fn classify(cell: &Cell) -> TerrainClass {
    // XXX: Hardcoded terrain name recognition. Non-spawning terrain that
    // matches a set of known names gets assigned as Floor or Wall. This can
    // affect glyph choice preference. What glyphs are used doesn't really
    // matter, but it can make the maps easier to read for humans.

    if cell.spawns.len() > 0 { return Spawning; }
    match cell.terrain_type.as_slice() {
        "floor"
      | "grass"
      | "sand"
      | "dirt"
        => Floor,
        "wall"
      | "rock"
      | "rock wall"
        => Wall,
        _ => Other
    }
}
