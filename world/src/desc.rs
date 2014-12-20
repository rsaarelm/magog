use calx::Rgb;

/// Entity name and appearance.
#[deriving(Clone, Show, Encodable, Decodable)]
pub struct Desc {
    pub name: String,
    pub icon: uint,
    pub color: Rgb,
}

impl Desc {
    pub fn new(name: String, icon: uint, color: Rgb) -> Desc {
        Desc {
            name: name,
            icon: icon,
            color: color,
        }
    }
}
