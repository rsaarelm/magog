use calx::KernelTerrain;
use self::TerrainType::*;

// TODO: Figure out how to not require explicit element count.
macro_rules! terrain_data {
    {
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

        pub static _TERRAINS: [TerrainType; count_exprs!($($name),*)] = [
            $($symbol,)*
            ];

    }
}

terrain_data! {
    Void, "void";
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
}


impl TerrainType {
    pub fn from_name(name: &str) -> Option<TerrainType> {
        for &t in _TERRAINS.iter() {
            if t.name() == name { return Some(t); }
        }
        None
    }

    pub fn blocks_sight(&self) -> bool {
        match *self {
            Wall | RockWall | Rock | Door | Tree | DeadTree => true,
            _ => false
        }
    }

    pub fn blocks_shot(&self) -> bool {
        match *self {
            Wall | RockWall | Rock | Tree | Stalagmite | Door | Menhir | DeadTree => true,
            _ => false
        }
    }

    pub fn blocks_walk(&self) -> bool {
        match *self {
            Floor | Shallows | Grass | Grass2 | Downstairs
                | Door | OpenDoor => false,
            _ => true
        }
    }

    pub fn is_exit(&self) -> bool {
        match *self {
            Downstairs => true,
            _ => false
        }
    }

    pub fn valid_spawn_spot(&self) -> bool { !self.blocks_walk() && !self.is_exit() }

    pub fn is_door(&self) -> bool { *self == Door }

    pub fn is_luminous(&self) -> bool { *self == Magma }

    pub fn name(&self) -> &'static str { terrain_name(*self) }
}

impl KernelTerrain for TerrainType {
    fn is_wall(&self) -> bool {
        match *self {
            Wall | RockWall | Rock | Door | OpenDoor | Window | Bars | Fence => true,
            _ => false
        }
    }

    fn is_block(&self) -> bool {
        match *self {
            Rock => true,
            _ => false
        }
    }
}
