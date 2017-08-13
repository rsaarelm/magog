use std::slice;

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

struct TerrainData {
    name: &'static str,
    kind: Kind,
    form: Form,
    map_chars: &'static str,
    /// For variants that should not show up in main terrain sets.
    is_irregular: bool,
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
    Empty:       TerrainData { name: "void",      kind: Kind::Block,  form: Form::Void,  map_chars: "",    is_irregular: false },
    Gate:        TerrainData { name: "gate",      kind: Kind::Ground, form: Form::Gate,  map_chars: ">",   is_irregular: false },
    Ground:      TerrainData { name: "ground",    kind: Kind::Ground, form: Form::Floor, map_chars: ".,_", is_irregular: false },
    Grass:       TerrainData { name: "grass",     kind: Kind::Ground, form: Form::Floor, map_chars: ",._", is_irregular: false },
    Water:       TerrainData { name: "water",     kind: Kind::Water,  form: Form::Floor, map_chars: "~=",  is_irregular: false },
    Magma:       TerrainData { name: "magma",     kind: Kind::Magma,  form: Form::Floor, map_chars: "=~",  is_irregular: false },
    Tree:        TerrainData { name: "tree",      kind: Kind::Block,  form: Form::Prop,  map_chars: "",    is_irregular: false },
    Wall:        TerrainData { name: "wall",      kind: Kind::Block,  form: Form::Wall,  map_chars: "#*",  is_irregular: false },
    Rock:        TerrainData { name: "rock",      kind: Kind::Block,  form: Form::Blob,  map_chars: "*#",  is_irregular: false },
    Door:        TerrainData { name: "door",      kind: Kind::Door,   form: Form::Wall,  map_chars: "|",   is_irregular: false },
    // TODO: Get rid of corridor, it only makes sense for mapgen bookkeeping and that doesn't
    // belong in persistent map.
    Corridor:    TerrainData { name: "ground",    kind: Kind::Ground, form: Form::Floor, map_chars: "_.,", is_irregular: true },
    OpenDoor:    TerrainData { name: "open door", kind: Kind::Ground, form: Form::Wall,  map_chars: "",    is_irregular: true },
    // TODO: Get rid of grass2, give render a coherent noise source for tiles and make it do the
    // variation locally.
    Grass2:      TerrainData { name: "grass",     kind: Kind::Ground, form: Form::Floor, map_chars: "",    is_irregular: true },
}

impl Terrain {
    pub fn iter() -> slice::Iter<'static, Terrain> { TERRAIN_ENUM.iter() }

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
            Kind::Ground | Kind::Corridor | Kind::Door => false,
            _ => true,
        }
    }

    pub fn name(self) -> &'static str { TERRAIN_DATA[self as usize].name }

    pub fn is_open(self) -> bool { self.kind() == Kind::Ground || self.kind() == Kind::Corridor }

    pub fn is_door(self) -> bool { self.kind() == Kind::Door }

    pub fn is_luminous(self) -> bool { self.kind() == Kind::Magma }

    pub fn is_wall(self) -> bool { self.form() == Form::Wall }

    pub fn is_hull(self) -> bool { self.form() == Form::Wall || self.form() == Form::Blob }

    pub fn is_blob(self) -> bool { self.form() == Form::Blob }

    pub fn is_block(self) -> bool { self.is_hull() || self.form() == Form::Prop }

    pub fn is_irregular(self) -> bool { TERRAIN_DATA[self as usize].is_irregular }

    /// For constructing text maps.
    pub fn preferred_map_chars(self) -> &'static str { TERRAIN_DATA[self as usize].map_chars }
}
