/*! Hex map display demo */

extern crate num;
extern crate tiled;
extern crate image;
extern crate calx;

use std::cell::{RefCell};
use std::collections::{HashMap};
use num::{Integer};
use calx::color::*;
use calx::backend::{CanvasBuilder, Canvas, CanvasUtil, Event, Key, SpriteCache, SpriteKey, Image};
use calx::{V2, Rect, IterTiles, color_key, Rgba};
use calx::{Projection, Kernel, KernelTerrain};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Spr {
    BlockNW,
    BlockN,
    BlockNE,
    BlockRock,
    BlockRock1,
    BlockRock2,

    FloorBlank,
    Floor,
    GrassFloor,
    WaterFloor,

    BrickWallShort,
    BrickWallShort1,
    BrickWall,
    BrickWall1,
    BrickWindowWall,
    BrickWindowWall1,
    BrickOpenWall,
    BrickOpenWall1,
    DoorWallShort,
    DoorWallShort1,
    DoorWall,
    DoorWall1,

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
                    BlockNW,
                    BlockN,
                    BlockNE,
                    BlockRock,
                    BlockRock1,
                    BlockRock2,
                  ]);

    ret.batch_add(builder, V2(-16, -22), V2(32, 32), &mut load(include_bytes!("assets/floors.png")),
                  vec![
                    FloorBlank,
                    Floor,
                    GrassFloor,
                    WaterFloor,
                  ]);

    let walls = vec![
        BrickWallShort,
        BrickWallShort1,
        BrickWall,
        BrickWall1,
        BrickWindowWall,
        BrickWindowWall1,
        BrickOpenWall,
        BrickOpenWall1,
        DoorWallShort,
        DoorWallShort1,
        DoorWall,
        DoorWall1,
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

thread_local!(static SPRITE_CACHE: RefCell<SpriteCache<Spr>> = RefCell::new(SpriteCache::new()));

fn init_sprite_cache(builder: &mut CanvasBuilder) {
    SPRITE_CACHE.with(|c| { *c.borrow_mut() = build_sprites(builder); });
}

fn spr(spr: Spr) -> Image {
    SPRITE_CACHE.with(|c| c.borrow().get(spr).expect("Sprite not found"))
}

fn spr_nth(spr: Spr, n: usize) -> Image {
    SPRITE_CACHE.with(|c| c.borrow().get_nth(spr, n).expect("Sprite not found"))
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

pub fn terrain_at(pos: V2<i32>) -> Terrain {
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

    let key = MAP.with(|m| V2(pos.0.mod_floor(&m.w), pos.1.mod_floor(&m.h)));

    match MAP.with(|m| m.terrain.get(&key).map(|&x| x)) {
        Some(t) => t,
        None => Terrain::Magma,
    }
}


trait RenderTerrain {
    fn render(&self, ctx: &mut Canvas, offset: V2<f32>);
}

impl RenderTerrain for Kernel<Terrain> {
    fn render(&self, ctx: &mut Canvas, offset: V2<f32>) {
        use Terrain::*;

        fn wallform<C: KernelTerrain>(
            k: &Kernel<C>, ctx: &mut Canvas, offset: V2<f32>,
            short: Spr, short_color: &Rgba, short_back: &Rgba,
            long: Spr, long_color: &Rgba, long_back: &Rgba) {
            let extends = k.wall_extends();
            if extends[0] {
                ctx.draw_image(spr_nth(long, 0), offset, 0.45, &(*long_color * 0.5), long_back);
            } else {
                ctx.draw_image(spr_nth(short, 0), offset, 0.45, &(*short_color * 0.5), short_back);
            }
            if extends[1] {
                ctx.draw_image(spr_nth(long, 1), offset, 0.45, long_color, long_back);
            } else {
                ctx.draw_image(spr_nth(short, 1), offset, 0.45, short_color, short_back);
            }
        }

        fn blockform<C: KernelTerrain>(
            k: &Kernel<C>, ctx: &mut Canvas, offset: V2<f32>,
            face: Spr, color: &Rgba, back: &Rgba) {
            ctx.draw_image(spr(Spr::FloorBlank), offset, 0.5, &BLACK, &BLACK);

            let faces = k.block_faces();

            if faces[5] { ctx.draw_image(spr(Spr::BlockNW), offset, 0.45, &(*color * 0.5), &BLACK); }
            if faces[0] { ctx.draw_image(spr(Spr::BlockN), offset, 0.45, &(*color * 0.5), &BLACK); }
            if faces[1] { ctx.draw_image(spr(Spr::BlockNE), offset, 0.45, &(*color * 0.5), &BLACK); }
            if faces[4] { ctx.draw_image(spr_nth(face, 0), offset, 0.45, &(*color * 0.25), &(*back * 0.25)); }
            if faces[3] { ctx.draw_image(spr_nth(face, 1), offset, 0.45, &(*color), &(*back)); }
            if faces[2] { ctx.draw_image(spr_nth(face, 2), offset, 0.45, &(*color * 0.5), &(*back * 0.5)); }
        }

        match self.center {
            Floor => ctx.draw_image(spr(Spr::Floor), offset, 0.5, &SLATEGRAY, &BLACK),
            Grass => ctx.draw_image(spr(Spr::GrassFloor), offset, 0.5, &DARKGREEN, &BLACK),
            Water => ctx.draw_image(spr(Spr::WaterFloor), offset, 0.5, &CYAN, &ROYALBLUE),
            Magma => ctx.draw_image(spr(Spr::WaterFloor), offset, 0.5, &YELLOW, &DARKRED),
            Tree => {
                ctx.draw_image(spr(Spr::Floor), offset, 0.5, &SLATEGRAY, &BLACK);
                ctx.draw_image(spr(Spr::TreeTrunk), offset, 0.45, &SADDLEBROWN, &BLACK);
                ctx.draw_image(spr(Spr::Foliage), offset, 0.45, &GREEN, &BLACK);
            }

            Wall => {
                ctx.draw_image(spr(Spr::Floor), offset, 0.5, &SLATEGRAY, &BLACK);
                wallform(self, ctx, offset,
                         Spr::BrickWallShort, &LIGHTSLATEGRAY, &BLACK,
                         Spr::BrickWall, &LIGHTSLATEGRAY, &BLACK);
            }

            Door => {
                ctx.draw_image(spr(Spr::Floor), offset, 0.5, &SLATEGRAY, &BLACK);
                wallform(self, ctx, offset,
                         Spr::BrickWallShort, &LIGHTSLATEGRAY, &BLACK,
                         Spr::BrickOpenWall, &LIGHTSLATEGRAY, &BLACK);
                wallform(self, ctx, offset,
                         Spr::DoorWallShort, &SADDLEBROWN, &BLACK,
                         Spr::DoorWall, &SADDLEBROWN, &BLACK);
            }

            Window => {
                ctx.draw_image(spr(Spr::Floor), offset, 0.5, &SLATEGRAY, &BLACK);
                wallform(self, ctx, offset,
                         Spr::BrickWallShort, &LIGHTSLATEGRAY, &BLACK,
                         Spr::BrickWindowWall, &LIGHTSLATEGRAY, &BLACK);
            }

            Rock => {
                blockform(self, ctx, offset, Spr::BlockRock, &DARKGOLDENROD, &BLACK);
            }
        }
    }
}

fn main() {
    let scroll_speed = 4f32;
    let mut screen_offset = V2(0.0f32, 0.0f32);
    let mut scroll_delta = V2(0.0f32, 0.0f32);

    let screen_rect = Rect(V2(0.0f32, 0.0f32), V2(640.0f32, 360.0f32));
    let mut builder = CanvasBuilder::new().set_size((screen_rect.1).0 as u32, (screen_rect.1).1 as u32);
    init_sprite_cache(&mut builder);
    let mut ctx = builder.build();

    loop {
        match ctx.next_event() {
            Event::RenderFrame => {
                screen_offset = screen_offset - scroll_delta;

                let proj = Projection::new(V2(16.0, 8.0), V2(-16.0, 8.0)).unwrap()
                    .view_offset(screen_offset);
                for pt in proj.inv_project_rectangle(&screen_rect).iter() {
                    Kernel::new(terrain_at, pt.map(|x| x as i32)).render(&mut ctx, proj.project(pt));
                }
            }

            Event::Quit => { return; }

            Event::KeyPressed(Key::Escape) => { return; }

            Event::KeyPressed(Key::F12) => {
                ctx.save_screenshot(&"hexworld");
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

            _ => {}
        }
    }
}
