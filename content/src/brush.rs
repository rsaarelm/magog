//////////// Brush cache boilerplate ////////////

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
    Logo,

    IconHeart,
    IconHalfHeart,
    IconNoHeart,
    IconShard,
    IconHalfShard,
    IconNoShard,

    BlockRear,
    BlockRock,

    BrickWall,
    BrickWindowWall,
    BrickOpenWall,
    DoorWall,
    BarsWall,
    FenceWall,
    HouseWall,
    RockWall,

    BlankFloor,
    Floor,
    Grass,
    Water,
    Shallows,

    Human,
    Snake,
    Dreg,
    Ogre,
    Wraith,
    Octopus,
    Bug,
    Ooze,
    Efreet,
    Serpent,
    SerpentMound,

    CursorBottom,
    CursorTop,
    Table,
    Stone,
    Fountain,
    Altar,
    Barrell,
    Stalagmite,
    Pillar,
    Grave,
    Crystal,
    Menhir,
    Sword,
    Helmet,
    Potion,
    Wand,
    Health,
    Knives,
    Armor,
    Ring,
    StairsDown,
    TreeTrunk,
    TreeFoliage,
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

    for (k, img) in vec![
        BlankFloor,
        Floor,
        Grass,
        Water,
        Shallows,
    ].into_iter().zip(
        builder.batch_add(offset, size, &mut load(include_bytes!("../assets/floors.png")))) {
        ret.insert(k, vec![img]);
    }

    for (k, img) in vec![
        Human,
        Snake,
        Dreg,
        Ogre,
        Wraith,
        Octopus,
        Bug,
        Ooze,
        Efreet,
        Serpent,
        SerpentMound,
    ].into_iter().zip(
        builder.batch_add(offset, size, &mut load(include_bytes!("../assets/mobs.png")))) {
        ret.insert(k, vec![img]);
    }

    for (k, img) in vec![
        CursorBottom,
        CursorTop,
        Table,
        Stone,
        Fountain,
        Altar,
        Barrell,
        Stalagmite,
        Pillar,
        Grave,
        Crystal,
        Menhir,
        Sword,
        Helmet,
        Potion,
        Wand,
        Health,
        Knives,
        Armor,
        Ring,
        StairsDown,
        TreeTrunk,
        TreeFoliage,
    ].into_iter().zip(
        builder.batch_add(offset, size, &mut load(include_bytes!("../assets/props.png")))) {
        ret.insert(k, vec![img]);
    }

    // Wall tiles.
    {
        let mut wall_images = Vec::new();
        let mut wall_sheet = load(include_bytes!("../assets/walls.png"));
        // Can't use batch_add for walls because the offsets alternate.
        for (i, rect) in wall_sheet.tiles(V2(16, 32)).take(28).enumerate() {
            let offset = V2(if i % 2 == 0 { -16 } else { 0 }, -16);
            let image = image::SubImage::new(&mut wall_sheet, rect.mn().0, rect.mn().1, rect.dim().0, rect.dim().1);
            wall_images.push(builder.add_image(offset, &image));
        }

        // Also I'm reusing the center wall piece a bit so I can't generalize
        // these either.

        ret.insert(BrickWall, vec![0, 1, 2, 3]          .into_iter().map(|x| wall_images[x]).collect());
        ret.insert(BrickWindowWall, vec![0, 1, 4, 5]    .into_iter().map(|x| wall_images[x]).collect());
        ret.insert(BrickOpenWall, vec![0, 1, 6, 7]      .into_iter().map(|x| wall_images[x]).collect());
        ret.insert(DoorWall, vec![8, 9, 10, 11]         .into_iter().map(|x| wall_images[x]).collect());
        ret.insert(BarsWall, vec![12, 13, 14, 15]       .into_iter().map(|x| wall_images[x]).collect());
        ret.insert(FenceWall, vec![16, 17, 18, 19]      .into_iter().map(|x| wall_images[x]).collect());
        ret.insert(HouseWall, vec![20, 21, 22, 23]      .into_iter().map(|x| wall_images[x]).collect());
        ret.insert(RockWall, vec![24, 25, 26, 27]       .into_iter().map(|x| wall_images[x]).collect());
    }

    // Block tiles
    {
        let mut block_sheet = load(include_bytes!("../assets/blocks.png"));

        for (y, brush) in vec![
            BlockRear,
            BlockRock,
        ].into_iter().enumerate() {
            // Another tricky setup. The first three tiles are 32x32 brushes,
            // the last four are 16x32.
            let mut frames = Vec::new();
            let y = y as u32;
            let mut x = 0u32;
            for &width in &[32, 32, 32, 16, 16, 16, 16] {
                let offset = (x as i32 % 32) - 16;
                frames.push(builder.add_image(V2(offset, -16),
                    &image::SubImage::new(&mut block_sheet, x, y * 32, width, 32)));
                x += width;
            }

            ret.insert(brush, frames)
        }
    }

    ret.insert(Logo, vec![builder.add_image(V2(0, 0), &load(include_bytes!("../assets/logo.png")))]);

    for (k, img) in vec![
        IconHeart,
        IconHalfHeart,
        IconNoHeart,
        IconShard,
        IconHalfShard,
        IconNoShard,
    ].into_iter().zip(
        builder.batch_add(V2(0, -8), V2(8, 8), &mut load(include_bytes!("../assets/icon.png")))) {
        ret.insert(k, vec![img]);
    }

    ret
}
