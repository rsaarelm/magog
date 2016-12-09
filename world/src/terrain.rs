use calx_resource::{Loadable, Resource};
use std::str::FromStr;
use brush::Brush;

/// Movement effect of a terrain tile.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Kind {
    /// Regular flat solid ground, can walk across easily.
    Ground,
    /// Like floor, but map generation treats it differently.
    Corridor,
    /// An obstacle that fills the entire cell, blocks field of view.
    Block,
    /// An obstacle that can be seen through.
    ///
    /// Not necessarily a literal window, thinner blocks like pillars and statues might also be
    /// see-through.
    Window,
    /// A tile that blocks sight but can be walked through.
    Door,
    /// Bodies of water, regular units can't walk into them.
    ///
    /// Flying units (if we have any) can cross. Falling into water is going to involve tricky
    /// logic since our maps aren't 3D.
    Water,
    /// Like water, but much more fun.
    Magma,
}

/// Visual form of a terrain tile.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Form {
    /// Nothingness, lets sight pass through portals
    Void,
    /// Marker pointing at adjacent nothingness
    Gate,
    /// Single frame on floor layer
    Floor,
    /// Single frame on object layer
    Prop,
    /// Blobbing form on object layer
    Blob,
    /// Wall-form on object layer
    Wall,
}

/// Data for the terrain in a single map cell.
#[derive(Clone, Debug)]
pub struct Tile {
    pub brush: Resource<Brush>,
    pub kind: Kind,
    pub form: Form,
}

impl Tile {
    pub fn new(brush: &str, kind: Kind, form: Form) -> Tile {
        Tile {
            brush: Resource::new(brush.to_string()).unwrap(),
            kind: kind,
            form: form,
        }
    }

    pub fn blocks_sight(&self) -> bool {
        match self.kind {
            Kind::Block | Kind::Door => true,
            _ => false,
        }
    }

    pub fn blocks_shot(&self) -> bool {
        match self.kind {
            Kind::Block | Kind::Window | Kind::Door => true,
            _ => false,
        }
    }

    pub fn blocks_walk(&self) -> bool {
        match self.kind {
            Kind::Ground | Kind::Corridor | Kind::Door => false,
            _ => true,
        }
    }

    pub fn is_open(&self) -> bool { self.kind == Kind::Ground || self.kind == Kind::Corridor }

    pub fn is_door(&self) -> bool { self.kind == Kind::Door }

    pub fn is_luminous(&self) -> bool { self.kind == Kind::Magma }

    pub fn is_wall(&self) -> bool { self.form == Form::Wall }

    pub fn is_hull(&self) -> bool { self.form == Form::Wall || self.form == Form::Blob }

    pub fn is_blob(&self) -> bool { self.form == Form::Blob }

    pub fn is_block(&self) -> bool { self.is_hull() || self.form == Form::Prop }
}

impl Loadable<u8> for Tile {}

impl_store!(TILE_STORE, u8, Tile);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, RustcEncodable, RustcDecodable)]
#[repr(u8)]
pub enum Id {
    Empty = 0,
    Gate,
    Ground,
    Grass,
    Water,
    Magma,
    Tree,
    Wall,
    Rock,
    Door,

    // XXX: Corridor is more a mapgen nicety than a thing that actually needs to be separate.
    Corridor,
    // XXX: OpenDoor and Grass2 are variants of Door and Grass, they shouldn't be in a set
    // where you pick terrains to paint from for example.
    OpenDoor,
    Grass2,
    // XXX: The ENUM_MAX pattern isn't very rustic, better idiom to iterate the terrain set?
    _MaxTerrain,
}

impl Id {
    pub fn from_u8(id: u8) -> Option<Id> {
        if id < Id::_MaxTerrain as u8 {
            Some(unsafe { ::std::mem::transmute::<u8, Id>(id) })
        } else {
            None
        }
    }
}

impl FromStr for Id {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::Id::*;

        for i in 0..(Id::_MaxTerrain as u8) {
            let id = Id::from_u8(i).expect("Couldn't turn u8 to terrain::Id");
            if &format!("{:?}", id) == s {
                return Ok(id);
            }
        }
        Err(format!("Unknown terrain '{}'", s))
    }
}
