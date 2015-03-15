use self::TerrainType::*;

// TODO: Figure out how to not require explicit element count.
macro_rules! terrain_data {
    {
        count: $count:expr;
        $($symbol:ident, $name:expr;)*
    } => {
#[derive(Copy, Eq, PartialEq, Clone, Debug)]
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

    Void, "void";
    Floor, "floor";
    Chasm, "chasm";
    Water, "water";
    Shallows, "shallows";
    Magma, "magma";
    Wall, "wall";
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
    Barrel, "barrel";
    Stone, "stone";
    DeadTree, "dead tree";
    TallGrass, "tall grass";
    CraterN, "crater";
    CraterNE, "crater";
    CraterSE, "crater";
    CraterS, "crater";
    CraterSW, "crater";
    CraterNW, "crater";
    Crater, "crater";
    Pod, "pod";
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
            Wall | Rock | Door | OpenDoor | Window => true,
            _ => false
        }
    }

    pub fn blocks_sight(self) -> bool {
        match self {
            Wall | Rock | Door | Tree | DeadTree | TallGrass => true,
            _ => false
        }
    }

    pub fn blocks_shot(self) -> bool {
        match self {
            Wall | Rock | Tree | Stalagmite | Door | DeadTree => true,
            _ => false
        }
    }

    pub fn blocks_walk(self) -> bool {
        match self {
            Floor | Shallows | Grass | Grass2 | Crater
                | CraterN | CraterNE | CraterSE
                | CraterS | CraterSW | CraterNW | Pod
                | Door | OpenDoor | TallGrass => false,
            _ => true
        }
    }

    pub fn is_exit(self) -> bool { false }

    pub fn valid_spawn_spot(self) -> bool { !self.blocks_walk() && !self.is_exit() }

    pub fn is_door(self) -> bool { self == Door }

    pub fn is_luminous(self) -> bool { self == Magma }

    pub fn is_hole(self) -> bool { self == Chasm }

    pub fn name(self) -> &'static str { terrain_name(self) }
}
