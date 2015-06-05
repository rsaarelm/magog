//////////// Sprite cache boilerplate ////////////

use std::ops::{Add};
use std::cell::{RefCell};
use calx::backend::{CanvasBuilder, SpriteCache, SpriteKey, Image};

impl SpriteKey for Spr { fn to_usize(self) -> usize { self as usize } }

impl Spr {
    /// Get the sprite image.
    pub fn get(self) -> Image {
        SPRITE_CACHE.with(|c| c.borrow().get(self).expect("Sprite not found"))
    }

    /// Build the actual sprites in a canvas for the enum set.
    pub fn init(builder: &mut CanvasBuilder) {
        SPRITE_CACHE.with(|c| { *c.borrow_mut() = build_sprites(builder); });
    }
}

impl Add<usize> for Spr {
    type Output = Spr;
    fn add(self, rhs: usize) -> Spr {
        // XXX: Assuming enum size is u8. The compiler should complain about this if it changes.
        let idx = self as u8 + rhs as u8;
        if idx >= Spr::MaxSpr as u8 { panic!("Invalid sprite offset"); }
        unsafe { ::std::mem::transmute(idx) }
    }
}

thread_local!(static SPRITE_CACHE: RefCell<SpriteCache<Spr>> = RefCell::new(SpriteCache::new()));

//////////// Custom definitions start here ////////////

#[derive(Copy, Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub enum Spr {
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
    Grunt,

    EdgeN,
    EdgeNE,
    EdgeSE,
    EdgeS,
    EdgeSW,
    EdgeNW,

    MaxSpr,
}

fn build_sprites(builder: &mut CanvasBuilder) -> SpriteCache<Spr> {
    use image;
    use calx::{V2, IterTiles, color_key, color, Rgba};

    use self::Spr::*;

    fn load(data: &'static [u8]) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        color_key(&image::load_from_memory(data).unwrap(), color::CYAN)
    }

    let mut ret = SpriteCache::new();

    ret.batch_add(builder, V2(-16, -16), V2(32, 32), &mut load(include_bytes!("../assets/blocks.png")),
                  vec![
                    BlockNW,
                    BlockN,
                    BlockNE,
                    BlockRock,
                    BlockRock1,
                    BlockRock2,
                  ]);

    ret.batch_add(builder, V2(-16, -16), V2(32, 32), &mut load(include_bytes!("../assets/floors.png")),
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

    let mut wall_sheet = load(include_bytes!("../assets/walls.png"));
    // Can't use batch_add for walls because the offsets alternate.
    for (i, rect) in wall_sheet.tiles(V2(16, 32)).take(walls.len()).enumerate() {
        let offset = V2(if i % 2 == 0 { -16 } else { 0 }, -16);
        let image = image::SubImage::new(&mut wall_sheet, rect.mn().0, rect.mn().1, rect.dim().0, rect.dim().1);
        ret.add(builder, walls[i], offset, &image);
    }

    ret.batch_add(builder, V2(-16, -16), V2(32, 32), &mut load(include_bytes!("../assets/props.png")),
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
                    Grunt,
                  ]);

    ret.batch_add(builder, V2(-16, -16), V2(32, 32), &mut load(include_bytes!("../assets/segments.png")),
                  vec![
                    EdgeN,
                    EdgeNE,
                    EdgeSE,
                    EdgeS,
                    EdgeSW,
                    EdgeNW,
                  ]);

    ret
}
