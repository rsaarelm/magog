/// Entity name and appearance.
#[deriving(Clone, Show, Encodable, Decodable)]
pub struct Desc {
    pub name: String,
    pub icon: uint,
}

impl Desc {
    pub fn new(name: String, icon: uint) -> Desc {
        Desc {
            name: name,
            icon: icon,
        }
    }
}
