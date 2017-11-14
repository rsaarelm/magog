use {Icon, FontData, ImageData};
use atlas_cache::{AtlasCache, SubImageSpec};
use brush::Brush;
use init;
use std::cell::RefCell;
use std::rc::Rc;
use vec_map::VecMap;
use vitral::ImageBuffer;
use world;

thread_local! {
    pub static ATLAS: RefCell<AtlasCache> = {
        let mut ret = AtlasCache::new(1024, 0);
        // XXX: Boilerplatey, would probably need a macro to clean this up, because include_bytes!
        // must resolve at compile time.
        ret.load_png("assets/blobs.png".to_string(), include_bytes!("../assets/blobs.png")).expect("Error loading blobs.png");
        ret.load_png("assets/floors.png".to_string(), include_bytes!("../assets/floors.png")).expect("Error loading floors.png");
        ret.load_png("assets/gui.png".to_string(), include_bytes!("../assets/gui.png")).expect("Error loading gui.png");
        ret.load_png("assets/logo.png".to_string(), include_bytes!("../assets/logo.png")).expect("Error loading logo.png");
        ret.load_png("assets/mobs.png".to_string(), include_bytes!("../assets/mobs.png")).expect("Error loading mobs.png");
        ret.load_png("assets/portals.png".to_string(), include_bytes!("../assets/portals.png")).expect("Error loading portals.png");
        ret.load_png("assets/props.png".to_string(), include_bytes!("../assets/props.png")).expect("Error loading props.png");
        ret.load_png("assets/splatter.png".to_string(), include_bytes!("../assets/splatter.png")).expect("Error loading splatter.png");
        ret.load_png("assets/walls.png".to_string(), include_bytes!("../assets/walls.png")).expect("Error loading walls.png");
        ret.add_sheet("solid".to_string(), ImageBuffer::from_fn(1, 1, |_, _| 0xffff_ffff));
        RefCell::new(ret)
    };

    static TERRAIN_BRUSHES: VecMap<Rc<Brush>> = init::terrain_brushes();
    static ENTITY_BRUSHES: VecMap<Rc<Brush>> = init::entity_brushes();
    static MISC_BRUSHES: VecMap<Rc<Brush>> = init::misc_brushes();
    static FONT: Rc<FontData> = Rc::new(
        init::font("font".to_string(), include_bytes!("../assets/font.png"), (32u8..128).map(|c| c as char)));
}

pub fn get(key: &SubImageSpec) -> ImageData { ATLAS.with(|a| a.borrow_mut().get(key).clone()) }

pub fn terrain(t: world::Terrain) -> Rc<Brush> {
    TERRAIN_BRUSHES.with(|b| {
        Rc::clone(b.get(t as usize).expect(
            &format!("No brush for terrain {:?}", t),
        ))
    })
}

pub fn entity(e: world::Icon) -> Rc<Brush> {
    ENTITY_BRUSHES.with(|b| {
        Rc::clone(b.get(e as usize).expect(
            &format!("No brush for entity {:?}", e),
        ))
    })
}

pub fn misc(e: Icon) -> Rc<Brush> {
    MISC_BRUSHES.with(|b| {
        Rc::clone(b.get(e as usize).expect(
            &format!("No brush for icon {:?}", e),
        ))
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
