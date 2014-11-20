use calx::Rgb;
use calx::color::*;
use {AreaSpec};
use {Biome};

/// Data component for mobs.
#[deriving(Clone, Show, Encodable, Decodable)]
pub struct Mob {
    pub max_hp: int,
    pub hp: int,
    pub power: int,
    pub armor: int,
    pub status: int,
}

impl Mob {
    pub fn new(t: MobType) -> Mob {
        let data = MOB_SPECS[t as uint];
        let status = if t != MobType::Player { Status::Asleep as int } else { 0 };
        Mob {
            max_hp: data.power,
            hp: data.power,
            power: data.power,
            armor: 0,
            status: status,
        }
    }

    pub fn has_status(&self, s: Status) -> bool {
        self.status as int & s as int != 0
    }

    pub fn add_status(&mut self, s: Status) {
        self.status |= s as int;
    }

    pub fn remove_status(&mut self, s: Status) {
        self.status &= !(s as int);
    }
}

#[deriving(Eq, PartialEq, Clone, Show, Encodable, Decodable)]
pub enum Intrinsic {
    /// Moves 1/3 slower than usual.
    Slow        = 0b1,
    /// Moves 1/3 faster than usual, stacks with Quick status.
    Fast        = 0b10,
    /// Can manipulate objects and doors.
    Hands       = 0b100,
}

#[deriving(Eq, PartialEq, Clone, Show, Encodable, Decodable)]
pub enum Status {
    /// Moves 1/3 slower than usual.
    Slow        = 0b1,
    /// Moves 1/3 faster than usual, stacks with Fast intrinsic.
    Quick       = 0b10,
    /// Mob is inactive until disturbed.
    Asleep      = 0b100,
    /// Mob moves erratically.
    Confused    = 0b1000,
}

pub struct MobSpec {
    pub typ: MobType,
    pub name: &'static str,
    pub power: int,
    pub area_spec: AreaSpec,
    pub sprite: uint,
    pub color: &'static Rgb,
    pub intrinsics: int,
}

// Intrinsic flag union.
macro_rules! f {
    { $($flag:ident),* } => { 0 $( | Intrinsic::$flag as int )* }
}

macro_rules! mob_data {
    {
        count: $count:expr;
        $($symbol:ident: $power:expr, $depth:expr, $biome:ident, $sprite:expr, $color:expr, $flags:expr;)*

    } => {
#[deriving(Eq, PartialEq, Clone, Show, Encodable, Decodable)]
pub enum MobType {
    $($symbol,)*
}

pub static MOB_SPECS: [MobSpec, ..$count] = [
    $(MobSpec {
        typ: MobType::$symbol,
        name: stringify!($symbol),
        power: $power,
        area_spec: AreaSpec {
            depth: $depth,
            biome: Biome::$biome,
        },
        sprite: $sprite,
        color: $color,
        intrinsics: $flags,
    },)*
];

// End macro
    }
}

mob_data! {
    count: 10;
//  Symbol   power, depth, biome, sprite, color,        intrinsics
    Player:     6,  -1, Anywhere, 51, &AZURE,            f!(Hands);
    Dreg:       1,   1, Anywhere, 72, &OLIVE,            f!(Hands);
    Snake:      1,   1, Overland, 71, &GREEN,            f!();
    Ooze:       3,   3, Dungeon,  77, &LIGHTSEAGREEN,    f!();
    Flayer:     4,   4, Anywhere, 75, &INDIANRED,        f!();
    Ogre:       6,   5, Anywhere, 73, &DARKSLATEGRAY,    f!(Hands);
    Wraith:     8,   6, Dungeon,  74, &HOTPINK,          f!(Hands);
    Octopus:    10,  7, Anywhere, 63, &DARKTURQUOISE,    f!();
    Efreet:     12,  8, Anywhere, 78, &ORANGE,           f!();
    Serpent:    15,  9, Dungeon,  94, &CORAL,            f!();
}
