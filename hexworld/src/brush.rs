//////////// Sprite cache boilerplate ////////////

use std::cell::{RefCell};
use calx::{ImageStore, IndexCache};
use calx::backend::{CanvasBuilder, Image};

cache_key!(Brush);

impl Brush {
    /// Get the sprite image.
    pub fn get(self, idx: usize) -> Image {
        BRUSH_CACHE.with(|c| c.borrow().get(self).expect("Brush not found")[idx])
    }

    /// Build the actual sprites in a canvas for the enum set.
    pub fn init(builder: &mut CanvasBuilder) {
        BRUSH_CACHE.with(|c| { *c.borrow_mut() = build_brushes(builder); });
    }
}

thread_local!(static BRUSH_CACHE: RefCell<IndexCache<Brush, Vec<Image>>> = RefCell::new(IndexCache::new()));

//////////// Custom definitions start here ////////////

#[derive(Copy, Clone, PartialEq, Eq, Debug, RustcEncodable, RustcDecodable)]
pub enum Brush {
    BlockRear,
    BlockRock,

    FloorBlank,
    Floor,
    GrassFloor,
    WaterFloor,

    BrickWall,
    BrickWindowWall,
    BrickOpenWall,
    DoorWall,

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

    Edge,
}

fn build_brushes(builder: &mut CanvasBuilder) -> IndexCache<Brush, Vec<Image>> {
    use image;
    use calx::{V2, IterTiles, color_key, color, Rgba};

    use self::Brush::*;

    fn load(data: &'static [u8]) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        color_key(&image::load_from_memory(data).unwrap(), color::CYAN)
    }

    let offset = V2(-16, -16);
    let size = V2(32, 32);

    let mut ret = IndexCache::new();

    {
        let blocks: Vec<Image> =
            builder.batch_add(offset, size, &mut load(include_bytes!("../assets/blocks.png"))).collect();
        ret.insert(BlockRear, blocks[0..3].to_vec());
        ret.insert(BlockRock, blocks[3..6].to_vec());
    }

    for (k, img) in vec![
        FloorBlank,
        Floor,
        GrassFloor,
        WaterFloor,
    ].into_iter().zip(
        builder.batch_add(offset, size, &mut load(include_bytes!("../assets/floors.png")))) {
        ret.insert(k, vec![img]);
    }


    // Wall tiles, this part is messy.
    {
        let mut wall_images = Vec::new();
        let mut wall_sheet = load(include_bytes!("../assets/walls.png"));
        // Can't use batch_add for walls because the offsets alternate.
        for (i, rect) in wall_sheet.tiles(V2(16, 32)).take(12).enumerate() {
            let offset = V2(if i % 2 == 0 { -16 } else { 0 }, -16);
            let image = image::SubImage::new(&mut wall_sheet, rect.mn().0, rect.mn().1, rect.dim().0, rect.dim().1);
            wall_images.push(builder.add_image(offset, &image));
        }

        ret.insert(BrickWall, vec![0, 1, 2, 3]          .into_iter().map(|x| wall_images[x]).collect());
        ret.insert(BrickWindowWall, vec![0, 1, 4, 5]    .into_iter().map(|x| wall_images[x]).collect());
        ret.insert(BrickOpenWall, vec![0, 1, 6, 7]      .into_iter().map(|x| wall_images[x]).collect());
        ret.insert(DoorWall, vec![8, 9, 10, 11]         .into_iter().map(|x| wall_images[x]).collect());
    }

    for (k, img) in vec![
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
    ].into_iter().zip(
        builder.batch_add(offset, size, &mut load(include_bytes!("../assets/props.png")))) {
        ret.insert(k, vec![img]);
    }

    ret.insert(Edge,
        builder.batch_add(offset, size, &mut load(include_bytes!("../assets/edges.png"))).collect());

    ret
}
