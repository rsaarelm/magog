use crate::brush::Brush;
use crate::init;
use crate::{AtlasCache, Icon, SubImageSpec};
use std::cell::RefCell;
use std::rc::Rc;
use vec_map::VecMap;
use vitral::{FontData, ImageBuffer, ImageData, PngBytes};
use world;

thread_local! {
    pub static ATLAS: RefCell<AtlasCache> = {
        let mut ret = AtlasCache::new(1024, 0);
        // All the asset images used by the game must be registered here.
        ret.add_sheet("assets/blobs.png", PngBytes(include_bytes!("../assets/blobs.png")));
        ret.add_sheet("assets/floors.png", PngBytes(include_bytes!("../assets/floors.png")));
        ret.add_sheet("assets/gui.png", PngBytes(include_bytes!("../assets/gui.png")));
        ret.add_sheet("assets/logo.png", PngBytes(include_bytes!("../assets/logo.png")));
        ret.add_sheet("assets/mobs.png", PngBytes(include_bytes!("../assets/mobs.png")));
        ret.add_sheet("assets/portals.png", PngBytes(include_bytes!("../assets/portals.png")));
        ret.add_sheet("assets/props.png", PngBytes(include_bytes!("../assets/props.png")));
        ret.add_sheet("assets/splatter.png", PngBytes(include_bytes!("../assets/splatter.png")));
        ret.add_sheet("assets/walls.png", PngBytes(include_bytes!("../assets/walls.png")));
        ret.add_sheet("solid", ImageBuffer::from_fn(1, 1, |_, _| 0xffff_ffff));
        RefCell::new(ret)
    };

    static TERRAIN_BRUSHES: VecMap<Rc<Brush>> = init::terrain_brushes();
    static ENTITY_BRUSHES: VecMap<Rc<Brush>> = init::entity_brushes();
    static MISC_BRUSHES: VecMap<Rc<Brush>> = init::misc_brushes();
    static FONT: Rc<FontData> = Rc::new(
        ATLAS.with(|a| a.borrow_mut().add_tilesheet_font("font", PngBytes(include_bytes!("../assets/font.png")),
                (32u8..128).map(|c| c as char))));
}

pub fn get(key: &SubImageSpec) -> ImageData { ATLAS.with(|a| a.borrow_mut().get(key).clone()) }

pub fn terrain(t: world::Terrain) -> Rc<Brush> {
    TERRAIN_BRUSHES.with(|b| {
        Rc::clone(
            b.get(t as usize)
                .expect(&format!("No brush for terrain {:?}", t)),
        )
    })
}

pub fn entity(e: world::Icon) -> Rc<Brush> {
    ENTITY_BRUSHES.with(|b| {
        Rc::clone(
            b.get(e as usize)
                .expect(&format!("No brush for entity {:?}", e)),
        )
    })
}

pub fn misc(e: Icon) -> Rc<Brush> {
    MISC_BRUSHES.with(|b| {
        Rc::clone(
            b.get(e as usize)
                .expect(&format!("No brush for icon {:?}", e)),
        )
    })
}

/// Return the single solid pixel texture for Vitral's graphics.
pub fn solid() -> ImageData {
    ATLAS.with(|a| {
        a.borrow_mut()
            .get(&SubImageSpec::new("solid", 0, 0, 1, 1))
            .clone()
    })
}

pub fn font() -> Rc<FontData> { FONT.with(|f| Rc::clone(f)) }
