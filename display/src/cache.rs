use std::cell::RefCell;
use std::rc::Rc;
use vec_map::VecMap;
use atlas_cache::{AtlasCache, SubImageSpec};
use world;
use brush::Brush;
use vitral;
use init;
use Icon;

thread_local! {
    pub static ATLAS: RefCell<AtlasCache> = {
        // TODO: Currently textures 0, 1 are baked in by Vitral, fix this.
        let mut ret = AtlasCache::new(1024, 2);
        // XXX: Boilerplatey, would probably need a macro to clean this up, because include_bytes!
        // must resolve at compile time.
        ret.load_tilesheet("assets/blobs.png".to_string(), include_bytes!("../assets/blobs.png")).expect("Error loading blobs.png");
        ret.load_tilesheet("assets/floors.png".to_string(), include_bytes!("../assets/floors.png")).expect("Error loading floors.png");
        ret.load_tilesheet("assets/logo.png".to_string(), include_bytes!("../assets/logo.png")).expect("Error loading logo.png");
        ret.load_tilesheet("assets/mobs.png".to_string(), include_bytes!("../assets/mobs.png")).expect("Error loading mobs.png");
        ret.load_tilesheet("assets/portals.png".to_string(), include_bytes!("../assets/portals.png")).expect("Error loading portals.png");
        ret.load_tilesheet("assets/props.png".to_string(), include_bytes!("../assets/props.png")).expect("Error loading props.png");
        ret.load_tilesheet("assets/splatter.png".to_string(), include_bytes!("../assets/splatter.png")).expect("Error loading splatter.png");
        ret.load_tilesheet("assets/walls.png".to_string(), include_bytes!("../assets/walls.png")).expect("Error loading walls.png");
        RefCell::new(ret)
    };

    static TERRAIN_BRUSHES: VecMap<Rc<Brush>> = init::terrain_brushes();
    static ENTITY_BRUSHES: VecMap<Rc<Brush>> = init::entity_brushes();
    static MISC_BRUSHES: VecMap<Rc<Brush>> = init::misc_brushes();
    static FONT: Rc<vitral::FontData<usize>> = Rc::new(
        init::font("font".to_string(), include_bytes!("../assets/font.png"), (32u8..128).map(|c| c as char)));
}

pub fn get(key: &SubImageSpec) -> vitral::ImageData<usize> {
    ATLAS.with(|a| a.borrow_mut().get(key).clone())
}

pub fn terrain(t: world::Terrain) -> Rc<Brush> {
    TERRAIN_BRUSHES.with(|b| {
        b.get(t as usize).expect(&format!("No brush for terrain {:?}", t)).clone()
    })
}

pub fn entity(e: world::Icon) -> Rc<Brush> {
    ENTITY_BRUSHES.with(|b| {
        b.get(e as usize).expect(&format!("No brush for entity {:?}", e)).clone()
    })
}

pub fn misc(e: Icon) -> Rc<Brush> {
    MISC_BRUSHES.with(|b| b.get(e as usize).expect(&format!("No brush for icon {:?}", e)).clone())
}

pub fn font() -> Rc<vitral::FontData<usize>> { FONT.with(|f| f.clone()) }
