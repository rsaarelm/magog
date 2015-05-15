/*! Hex map display demo */

extern crate num;
extern crate tiled;
extern crate image;
extern crate calx;

use std::collections::{HashMap};
use num::{Integer};
use calx::color::*;
use calx::backend::{CanvasBuilder, CanvasUtil, Event, Key, SpriteCache, SpriteKey};
use calx::{V2, Rect, IterTiles, color_key, Projection};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Spr {
    BlockRear,
    BlockRear1,
    BlockRear2,
    BlockFront,
    BlockFront1,
    BlockFront2,

    FloorBlank,
    Floor,
    GrassFloor,
    WaterFloor,
    PentagramFloor,
    GridFloor,

    BrickWall,
    BrickWall1,
    BrickWall2,
    BrickWall3,
    BrickWindowWall,
    BrickWindowWall1,
    BrickWindowWall2,
    BrickWindowWall3,
    BrickOpenWall,
    BrickOpenWall1,
    BrickOpenWall2,
    BrickOpenWall3,
    HouseWall,
    HouseWall1,
    HouseWall2,
    HouseWall3,
    HouseWindowWall,
    HouseWindowWall1,
    HouseWindowWall2,
    HouseWindowWall3,
    HouseOpenWall,
    HouseOpenWall1,
    HouseOpenWall2,
    HouseOpenWall3,
    Door,
    Door1,
    Door2,
    Door3,

    TreeTrunk,
    Foliage,
    Table,
    Avatar,
    Fountain,
    Altar,
    Barrel,
    Stalagmite,
    Pillar,
    Grave,
    Crystal,
    Menhir,
}

impl SpriteKey for Spr { fn to_usize(self) -> usize { self as usize } }

fn build_sprites(builder: &mut CanvasBuilder) -> SpriteCache<Spr> {
    use Spr::*;

    fn load(data: &'static [u8]) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        calx::color_key(&image::load_from_memory(data).unwrap(), &CYAN)
    }

    let mut ret = SpriteCache::new();

    ret.batch_add(builder, V2(-16, -22), V2(32, 32), &mut load(include_bytes!("assets/blocks.png")),
                  vec![
                    BlockRear,
                    BlockRear1,
                    BlockRear2,
                    BlockFront,
                    BlockFront1,
                    BlockFront2,
                  ]);

    ret.batch_add(builder, V2(-16, -22), V2(32, 32), &mut load(include_bytes!("assets/floors.png")),
                  vec![
                    FloorBlank,
                    Floor,
                    GrassFloor,
                    WaterFloor,
                    PentagramFloor,
                    GridFloor,
                  ]);

    let walls = vec![
        BrickWall,
        BrickWall1,
        BrickWall2,
        BrickWall3,
        BrickWindowWall,
        BrickWindowWall1,
        BrickWindowWall2,
        BrickWindowWall3,
        BrickOpenWall,
        BrickOpenWall1,
        BrickOpenWall2,
        BrickOpenWall3,
        HouseWall,
        HouseWall1,
        HouseWall2,
        HouseWall3,
        HouseWindowWall,
        HouseWindowWall1,
        HouseWindowWall2,
        HouseWindowWall3,
        HouseOpenWall,
        HouseOpenWall1,
        HouseOpenWall2,
        HouseOpenWall3,
        Door,
        Door1,
        Door2,
        Door3,
    ];

    let mut wall_sheet = load(include_bytes!("assets/walls.png"));
    // Can't use batch_add for walls because the offsets alternate.
    for (i, rect) in wall_sheet.tiles(V2(16, 32)).take(walls.len()).enumerate() {
        let offset = V2(if i % 2 == 0 { -16 } else { 0 }, -22);
        let image = image::SubImage::new(&mut wall_sheet, rect.mn().0, rect.mn().1, rect.dim().0, rect.dim().1);
        ret.add(builder, walls[i], offset, &image);
    }

    ret.batch_add(builder, V2(-16, -22), V2(32, 32), &mut load(include_bytes!("assets/props.png")),
                  vec![
                    TreeTrunk,
                    Foliage,
                    Table,
                    Avatar,
                    Fountain,
                    Altar,
                    Barrel,
                    Stalagmite,
                    Pillar,
                    Grave,
                    Crystal,
                    Menhir,
                  ]);

    ret
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Terrain {
    Floor,
    Grass,
    Water,
    Tree,
    Wall,
    Door,
    Window,
    Magma,
    HouseWall,
    HouseDoor,
    HouseWindow,
    Rock,
}

impl Terrain {
    pub fn new(id: u8) -> Terrain {
        // Tiled indexes start from 1.
        let id = id - 1;
        assert!(id <= Terrain::Rock as u8);
        unsafe {
            std::mem::transmute(id)
        }
    }
}

impl HexTerrain for Terrain {
    fn is_wall(&self) -> bool {
        use Terrain::*;
        match *self {
            Wall | Door | Window | HouseWall | HouseDoor | HouseWindow => true,
            _ => false
        }
    }

    fn is_block(&self) -> bool { *self == Terrain::Rock }
}

/// Shaping properties for hex terrain cells.
pub trait HexTerrain {
    /// Terrain is a wall with thin, shaped pieces along the (1, 0) and (0, 1) hex axes.
    fn is_wall(&self) -> bool;

    /// Terrain is a solid block that fills the entire hex.
    fn is_block(&self) -> bool;

    /// Terrain is either a wall or a block.
    fn is_hull(&self) -> bool { self.is_wall() || self.is_block() }
}

fn load_tmx_map() -> (u32, u32, HashMap<V2<i32>, Terrain>) {
    let tmx = include_str!("assets/hexworld.tmx");
    let map = tiled::parse(tmx.as_bytes()).unwrap();
    let mut ret = HashMap::new();

    let (w, h) = (map.width, map.height);
    for layer in map.layers.iter() {
        for (y, row) in layer.tiles.iter().enumerate() {
            for (x, &id) in row.iter().enumerate() {
                ret.insert(V2(x as i32, y as i32), Terrain::new(id as u8));
            }
        }
    }

    (w, h, ret)
}

pub fn terrain_at(pos: V2<f32>) -> Terrain {
    struct Map {
        w: i32,
        h: i32,
        terrain: HashMap<V2<i32>, Terrain>,
    }

    // Tiled map data as the backend.
    thread_local!(static MAP: Map = {
        let (w, h, terrain) = load_tmx_map();
        Map { w: w as i32, h: h as i32, terrain: terrain }
    });

    let pos = pos.map(|x| x as i32);

    let key = MAP.with(|m| V2(pos.0.mod_floor(&m.w), pos.1.mod_floor(&m.h)));

    match MAP.with(|m| m.terrain.get(&key).map(|&x| x)) {
        Some(t) => t,
        None => Terrain::Magma,
    }
}

fn main() {
    let screen_rect = Rect(V2(0.0f32, 0.0f32), V2(640.0f32, 360.0f32));
    let mut builder = CanvasBuilder::new().set_size((screen_rect.1).0 as u32, (screen_rect.1).1 as u32);
    let cache = build_sprites(&mut builder);
    let mut ctx = builder.build();

    loop {
        match ctx.next_event() {
            Event::RenderFrame => {
                let proj = Projection::new(V2(16.0, 8.0), V2(-16.0, 8.0)).unwrap();
                for pt in proj.inv_project_rectangle(&screen_rect).iter() {
                    let terrain = terrain_at(pt);
                    let draw_pos = proj.project(pt);
                    match terrain {
                        Terrain::Grass => {
                            ctx.draw_image(cache.get(Spr::GrassFloor).unwrap(), draw_pos, 0.5, &GREEN, &BLACK);
                        }
                        _ => {} // TODO
                    }
                }
            }

            Event::Quit => { return; }

            Event::KeyPressed(Key::Escape) => { return; }

            Event::KeyPressed(Key::F12) => {
                ctx.save_screenshot(&"hexworld");
            }

            _ => {}
        }
    }
}
