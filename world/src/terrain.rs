use self::TerrainType::*;

// TODO: Figure out how to not require explicit element count.
macro_rules! terrain_data {
    {
        count: $count:expr;
        $($symbol:ident, $name:expr;)*
    } => {
#[derive(Copy, Eq, PartialEq, Clone, Show)]
        pub enum TerrainType {
            $($symbol,)*
        }

        fn terrain_name(t: TerrainType) -> &'static str {
            match t {
                $($symbol => $name,)*
            }
        }

        pub static TERRAINS: [TerrainType; $count] = [
            $($symbol,)*
            ];

    }
}

terrain_data! {
    count: 28;

    Space, "space";
    Floor, "floor";
    Water, "water";
    Shallows, "shallows";
    Magma, "magma";
    Downstairs, "stairs down";
    Wall, "wall";
    RockWall, "rock wall";
    Rock, "rock";
    Tree, "tree";
    Grass, "grass";
    // Render variant, do not use directly.
    Grass2, "grass";
    Stalagmite, "stalagmite";
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
    Battlement, "battlement";
}

impl TerrainType {
    pub fn from_name(name: &str) -> Option<TerrainType> {
        for &t in TERRAINS.iter() {
            if t.name() == name { return Some(t); }
        }
        None
    }

    /// Is this terrain a solid block of mass? Hull terrain gets shaped based
    /// on its surroundings when drawn.
    pub fn is_hull(self) -> bool { self.is_block() || self.is_wall() }

    /// Is this terrain a natural landscape hull that gets shaped hexagonally?
    pub fn is_block(self) -> bool {
        match self {
            // TODO: Figure out the rest of the terrains.
            Rock
                => true,
            _ => false
        }
    }

    pub fn is_wall(self) -> bool {
        match self {
            Wall | RockWall | Door | OpenDoor | Window |
                Bars | Fence | Battlement => true,
            _ => false
        }
    }

    pub fn blocks_sight(self) -> bool {
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

    pub fn blocks_walk(self) -> bool {
        match self {
            Floor | Shallows | Grass | Grass2 | Downstairs
                | Door | OpenDoor | TallGrass | Battlement => false,
            _ => true
        }
    }

    pub fn is_exit(self) -> bool {
        match self {
            Downstairs => true,
            _ => false
        }
    }

    pub fn is_door(self) -> bool { self == Door }

    pub fn is_luminous(self) -> bool { self == Magma }

    pub fn is_space(self) -> bool { self == Space }

    pub fn is_ground(self) -> bool {
        // TODO: Liquids are hulls but not ground.
        self.is_hull()
    }

    pub fn name(self) -> &'static str { terrain_name(self) }
}
