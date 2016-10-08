use calx_resource::{Loadable, Resource};
use brush::Brush;

/// Movement effect of a terrain tile.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Kind {
    /// Regular flat solid ground, can walk across easily.
    Ground,
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
#[derive(Copy, Clone, Eq, PartialEq)]
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
#[derive(Clone)]
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
            Kind::Ground | Kind::Door => false,
            _ => true,
        }
    }

    pub fn is_door(&self) -> bool { self.kind == Kind::Door }

    pub fn is_luminous(&self) -> bool { self.kind == Kind::Magma }

    pub fn is_wall(&self) -> bool { self.form == Form::Wall }

    pub fn is_hull(&self) -> bool { self.form == Form::Wall || self.form == Form::Blob }

    pub fn is_block(&self) -> bool { self.is_hull() || self.form == Form::Prop }
}

impl Loadable<u8> for Tile {}

impl_store!(TILE_STORE, u8, Tile);

pub enum Id {
    Empty = 0,
    Gate = 1,
    Ground = 2,
    Grass = 3,
    Water = 4,
    Tree = 5,
    Wall = 6,
    Rock = 7,
}
