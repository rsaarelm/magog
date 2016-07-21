use calx_resource::Resource;
use brush::Brush;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Kind {
    Ground,
    Block,
    Water,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Form {
    /// Single frame on floor layer
    Floor,
    /// Single frame on object layer
    Prop,
    /// Block-form on object layer
    Block,
    /// Wall-form on object layer
    Wall,
}

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
}
