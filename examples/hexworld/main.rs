/*! Hex map display demo */

extern crate num;
extern crate tiled;
extern crate image;
extern crate calx;

mod spr;
mod render;
mod world;

use std::collections::{HashMap};
use calx::backend::{CanvasBuilder, CanvasUtil, Event, Key};
use calx::{V2, Rect, Rgba, color, ToColor, convert_color};
use calx::{Projection, Kernel, KernelTerrain};
use calx::{Dir6, LatticeNode, HexGeom, astar_path_with};

use spr::Spr;
use render::RenderTerrain;
use world::World;

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
    Rock,
    Void,
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

    pub fn can_walk(self) -> bool {
        use Terrain::*;
        match self {
            Floor | Grass | Door => true,
            _ => false,
        }
    }
}

impl KernelTerrain for Terrain {
    fn is_wall(&self) -> bool {
        use Terrain::*;
        match *self {
            Wall | Door | Window => true,
            _ => false
        }
    }

    fn is_block(&self) -> bool { *self == Terrain::Rock }
}

fn load_tmx_map() -> (u32, u32, HashMap<V2<i32>, Terrain>) {
    let tmx = include_str!("../assets/hexworld.tmx");
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

pub fn terrain_at(pos: V2<i32>) -> Terrain {
    struct Map {
        _w: i32,
        _h: i32,
        terrain: HashMap<V2<i32>, Terrain>,
    }

    // Tiled map data as the backend.
    thread_local!(static MAP: Map = {
        let (w, h, terrain) = load_tmx_map();
        Map { _w: w as i32, _h: h as i32, terrain: terrain }
    });

    //let key = MAP.with(|m| V2(pos.0.mod_floor(&m.w), pos.1.mod_floor(&m.h)));
    let key = pos;

    match MAP.with(|m| m.terrain.get(&key).map(|&x| x)) {
        Some(t) => t,
        None => Terrain::Void,
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, PartialOrd, Ord)]
struct PathPos(pub V2<i32>);

impl LatticeNode for PathPos {
    fn neighbors(&self) -> Vec<PathPos> {
        let mut ret = Vec::new();
        for i in Dir6::iter() {
            let pos = self.0 + i.to_v2();
            if terrain_at(pos).can_walk() {
                ret.push(PathPos(pos));
            }
        }
        ret
    }
}

struct Sprite {
    pub spr: Spr,
    pub fore: Rgba,
    pub back: Rgba,
    pub pos: V2<f32>,
    /// Draw layer, y coordinate.
    pub sort_key: (i8, i32),
}

impl Sprite {
    pub fn new<A: ToColor, B: ToColor>(spr: Spr, pos: V2<f32>, layer: i8, fore: &A, back: &B) -> Sprite {
        Sprite {
            spr: spr,
            fore: convert_color(fore),
            back: convert_color(back),
            pos: pos,
            sort_key: (layer, -pos.1 as i32)
        }
    }
}

fn main() {
    let scroll_speed = 4f32;
    let mut screen_offset = V2(0.0f32, 0.0f32);
    let mut scroll_delta = V2(0.0f32, 0.0f32);
    let mut mouse_pos = V2(-1.0f32, -1.0f32);

    let world = World::new();

    let screen_rect = Rect(V2(0.0f32, 0.0f32), V2(640.0f32, 360.0f32));
    let mut builder = CanvasBuilder::new().set_size((screen_rect.1).0 as u32, (screen_rect.1).1 as u32);
    Spr::init(&mut builder);
    let mut ctx = builder.build();

    loop {
        match ctx.next_event() {
            Event::RenderFrame => {
                screen_offset = screen_offset - scroll_delta;

                let mut sprites = Vec::new();

                let proj = Projection::new(V2(16.0, 8.0), V2(-16.0, 8.0)).unwrap()
                    .view_offset(screen_offset);
                for pt in proj.inv_project_rectangle(&screen_rect).iter() {
                    let pos = proj.project(pt);
                    Kernel::new(terrain_at, pt.map(|x| x as i32)).render(
                        |layer, spr, fore, back| {
                            sprites.push(Sprite::new(spr, pos, layer, fore, back));
                        });
                }

                let mouse_cell = PathPos(proj.inv_project(mouse_pos).map(|x| x.floor() as i32));
                let player_cell = PathPos(world.player_pos.map(|x| x.floor() as i32));
                let path = astar_path_with(|x, y| (x.0-y.0).hex_dist(), player_cell, mouse_cell, 1000);

                if let Some(path) = path {
                    for p in path.iter() {
                        sprites.push(Sprite::new(
                            Spr::Avatar, proj.project(p.0.map(|x| x as f32)), -1, &"#ff000088", &color::BLACK));
                    }
                }

                sprites.push(Sprite::new(Spr::Avatar, proj.project(world.player_pos), 0, &color::WHITE, &color::BLACK));

                sprites.sort_by(|a, b| b.sort_key.partial_cmp(&a.sort_key).unwrap());
                for spr in sprites.iter() {
                    ctx.draw_image(spr.spr.get(), spr.pos, 0.5, &spr.fore, &spr.back);
                }
            }

            Event::Quit => { return; }

            Event::KeyPressed(Key::Escape) => { return; }

            Event::KeyPressed(Key::F12) => {
                ctx.save_screenshot(&"azag");
            }

            Event::KeyPressed(k) => {
                match k {
                    Key::A => { scroll_delta.0 = -1.0 * scroll_speed; }
                    Key::D => { scroll_delta.0 =  1.0 * scroll_speed; }
                    Key::W => { scroll_delta.1 = -1.0 * scroll_speed; }
                    Key::S => { scroll_delta.1 =  1.0 * scroll_speed; }
                    _ => {}
                }
            }

            Event::KeyReleased(k) => {
                match k {
                    Key::A => { scroll_delta.0 = 0.0; }
                    Key::D => { scroll_delta.0 = 0.0; }
                    Key::W => { scroll_delta.1 = 0.0; }
                    Key::S => { scroll_delta.1 = 0.0; }
                    _ => {}
                }
            }
            Event::MouseMoved((x, y)) => {
                mouse_pos = V2(x, y).map(|x| x as f32);
            }

            _ => {}
        }
    }
}
