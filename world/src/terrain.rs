use serde_derive::{Deserialize, Serialize};
use std::slice;
use vitral::SRgba;

/// Movement effect of a terrain tile.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

struct TerrainData {
    name: &'static str,
    kind: Kind,
    form: Form,
    map_chars: &'static str,
    /// For variants that should not show up in main terrain sets.
    is_regular: bool,
    /// 4-bit components, R << 8 + G << 4 + B.
    color: u16,
}

macro_rules! count_tts {
    () => {0usize};
    ($_head:tt $($tail:tt)*) => {1usize + count_tts!($($tail)*)};
}

macro_rules! terrain_enum {
    {
        $($sym:ident: $data:expr,)+
    } => {
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
        pub enum Terrain {
            $($sym,)+
        }

        const N_ENUM: usize = count_tts!($($sym)+);

        static TERRAIN_DATA: [TerrainData; N_ENUM] = [
            $($data,)+
        ];

        static TERRAIN_ENUM: [Terrain; N_ENUM] = [
            $(Terrain::$sym,)+
        ];
    }
}

terrain_enum! {
    Empty:       TerrainData { name: "void",      kind: Kind::Block,  form: Form::Void,  map_chars: "",    is_regular: true,  color: 0xF0F },
    Exit:        TerrainData { name: "exit",      kind: Kind::Ground, form: Form::Gate,  map_chars: ">",   is_regular: true,  color: 0x0EE },
    Entrance:    TerrainData { name: "entrance",  kind: Kind::Ground, form: Form::Gate,  map_chars: "<",   is_regular: true,  color: 0x0FF },
    Ground:      TerrainData { name: "ground",    kind: Kind::Ground, form: Form::Floor, map_chars: ".,_", is_regular: true,  color: 0x111 },
    Grass:       TerrainData { name: "grass",     kind: Kind::Ground, form: Form::Floor, map_chars: ",._", is_regular: true,  color: 0x231 },
    Sand:        TerrainData { name: "sand",      kind: Kind::Ground, form: Form::Floor, map_chars: ",._", is_regular: true,  color: 0x650 },
    Snow:        TerrainData { name: "snow",      kind: Kind::Ground, form: Form::Floor, map_chars: ",._", is_regular: true,  color: 0x788 },
    Water:       TerrainData { name: "water",     kind: Kind::Water,  form: Form::Floor, map_chars: "~=",  is_regular: true,  color: 0x058 },
    Shallows:    TerrainData { name: "shallows",  kind: Kind::Ground, form: Form::Floor, map_chars: "~=",  is_regular: true,  color: 0x08B },
    Magma:       TerrainData { name: "magma",     kind: Kind::Magma,  form: Form::Floor, map_chars: "=~",  is_regular: true,  color: 0xF22 },
    Tree:        TerrainData { name: "tree",      kind: Kind::Block,  form: Form::Prop,  map_chars: "",    is_regular: true,  color: 0x8B1 },
    Wall:        TerrainData { name: "wall",      kind: Kind::Block,  form: Form::Wall,  map_chars: "#*",  is_regular: true,  color: 0xBBB },
    Rock:        TerrainData { name: "rock",      kind: Kind::Block,  form: Form::Blob,  map_chars: "*#",  is_regular: true,  color: 0xB84 },
    Door:        TerrainData { name: "door",      kind: Kind::Door,   form: Form::Wall,  map_chars: "|",   is_regular: true,  color: 0x842 },
    OpenDoor:    TerrainData { name: "open door", kind: Kind::Ground, form: Form::Wall,  map_chars: "",    is_regular: false, color: 0xFAF },
    Window:      TerrainData { name: "window",    kind: Kind::Window, form: Form::Wall,  map_chars: "+",   is_regular: true,  color: 0xBFF },
    Pillar:      TerrainData { name: "pillar",    kind: Kind::Block,  form: Form::Prop,  map_chars: "I",   is_regular: true,  color: 0xCCD },
    // TODO: Get rid of grass2, give render a coherent noise source for tiles and make it do the
    // variation locally.
    Grass2:      TerrainData { name: "grass",     kind: Kind::Ground, form: Form::Floor, map_chars: "",    is_regular: false, color: 0x230 },
}

impl Terrain {
    pub fn iter() -> slice::Iter<'static, Terrain> { TERRAIN_ENUM.iter() }

    pub fn from_color(color: SRgba) -> Option<Terrain> {
        let key =
            (((color.r >> 4) as u16) << 8) + (((color.g >> 4) as u16) << 4) + (color.b >> 4) as u16;
        Self::iter()
            .filter(|t| t.is_regular())
            .find(|t| TERRAIN_DATA[**t as usize].color == key)
            .cloned()
    }

    #[inline(always)]
    pub fn kind(self) -> Kind { TERRAIN_DATA[self as usize].kind }

    #[inline(always)]
    pub fn form(self) -> Form { TERRAIN_DATA[self as usize].form }

    pub fn blocks_sight(self) -> bool {
        match self.kind() {
            Kind::Block | Kind::Door => true,
            _ => false,
        }
    }

    pub fn blocks_shot(self) -> bool {
        match self.kind() {
            Kind::Block | Kind::Window | Kind::Door => true,
            _ => false,
        }
    }

    pub fn blocks_walk(self) -> bool {
        match self.kind() {
            Kind::Ground | Kind::Door => false,
            _ => true,
        }
    }

    pub fn name(self) -> &'static str { TERRAIN_DATA[self as usize].name }

    pub fn is_open(self) -> bool { self.kind() == Kind::Ground }

    pub fn is_door(self) -> bool { self.kind() == Kind::Door }

    pub fn is_luminous(self) -> bool { self.kind() == Kind::Magma }

    pub fn is_wall(self) -> bool { self.form() == Form::Wall }

    pub fn is_hull(self) -> bool { self.form() == Form::Wall || self.form() == Form::Blob }

    pub fn is_blob(self) -> bool { self.form() == Form::Blob }

    pub fn is_block(self) -> bool { self.is_hull() || self.form() == Form::Prop }

    pub fn is_regular(self) -> bool { TERRAIN_DATA[self as usize].is_regular }

    /// For constructing text maps.
    pub fn preferred_map_chars(self) -> &'static str { TERRAIN_DATA[self as usize].map_chars }

    /// Terrain is a narrow object that blocks movement.
    ///
    /// Prop obstacles might not be distinguishable from floors if you only see a corner of the
    /// terrain tile. Use this if there's need to highlight partially visible terrain as obstacles.
    pub fn is_narrow_obstacle(self) -> bool { self.blocks_walk() && self.form() == Form::Prop }

    pub fn color(self) -> SRgba {
        let col = TERRAIN_DATA[self as usize].color;
        let r = ((col >> 8) & 0xf) as u8;
        let g = ((col >> 4) & 0xf) as u8;
        let b = (col & 0xf) as u8;

        SRgba::new(r << 4, g << 4, b << 4, 0xff)
    }
}

impl Default for Terrain {
    fn default() -> Self { Terrain::Empty }
}

#[cfg(test)]
mod test {
    use super::*;
    use calx::SRgba;

    #[test]
    fn test_colors_are_unique() {
        use std::collections::HashSet;

        let terrains: HashSet<Terrain> = Terrain::iter()
            .filter(|t| t.is_regular())
            .cloned()
            .collect();
        let colors: HashSet<SRgba> = terrains.iter().map(|t| t.color()).collect();

        assert_eq!(colors.len(), terrains.len());
    }

    #[test]
    fn test_from_color() {
        assert_eq!(
            Terrain::from_color(SRgba::new(0xff, 0x88, 0xff, 0xff)),
            None
        );
        assert_eq!(
            Terrain::from_color(SRgba::new(0x22, 0x30, 0x1f, 0xff)),
            Some(Terrain::Grass)
        );
    }
}
