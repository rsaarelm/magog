// TODO: Figure out how to not require explicit element count.
macro_rules! terrain_data {
    {
        count: $count:expr;
        $($symbol:ident, $name:expr;)*
    } => {
#[deriving(Eq, PartialEq, Clone, Show)]
        pub enum TerrainType {
            $($symbol,)*
        }

        fn terrain_name(t: TerrainType) -> &'static str {
            match t {
                $($symbol => $name,)*
            }
        }

        pub static TERRAINS: [TerrainType, ..$count] = [
            $($symbol,)*
            ];

    }
}

terrain_data! {
    count: 28;

    Void, "void";
    Floor, "floor";
    Chasm, "chasm";
    Water, "water";
    Shallows, "shallows";
    Magma, "magma";
    Downstairs, "stairs down";
    Wall, "wall";
    RockWall, "rock wall";
    Rock, "rock";
    Tree, "tree";
    Grass, "grass";
    Stalagmite, "stalagmite";
    Portal, "portal";
    Door, "door";
    OpenDoor, "open door";
    Window, "window";
    Table, "table";
    Fence, "fence";
    Bars, "bars";
    Fountain, "fountain";
    Altar, "altar";
    Barrel, "barrel";
    Grave, "grave";
    Stone, "stone";
    Menhir, "menhir";
    DeadTree, "dead tree";
    TallGrass, "tall grass";
}


impl TerrainType {
    pub fn from_name(name: &str) -> Option<TerrainType> {
        for &t in TERRAINS.iter() {
            if t.name() == name { return Some(t); }
        }
        None
    }

    pub fn is_wall(self) -> bool {
        match self {
            Wall | RockWall | Rock | Door | OpenDoor | Window | Bars | Fence => true,
            _ => false
        }
    }

    pub fn is_opaque(self) -> bool {
        match self {
            Wall | RockWall | Rock | Door | Tree | DeadTree | TallGrass => true,
            _ => false
        }
    }

    pub fn blocks_shot(self) -> bool {
        match self {
            Wall | RockWall | Rock | Tree | Stalagmite | Door | Menhir | DeadTree => true,
            _ => false
        }
    }

    pub fn is_walkable(self) -> bool {
        match self {
            Floor | Shallows | Grass | Downstairs | Portal
                | Door | OpenDoor | TallGrass => true,
            _ => false
        }
    }

    pub fn is_exit(self) -> bool {
        match self {
            Downstairs => true,
            _ => false
        }
    }

    pub fn name(self) -> &'static str { terrain_name(self) }
}
