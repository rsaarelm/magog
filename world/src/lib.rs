extern crate num;
extern crate rand;
extern crate bincode;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate vec_map;
extern crate euclid;
#[macro_use]
extern crate lazy_static;
extern crate toml;
#[macro_use]
extern crate error_chain;
extern crate calx_alg;
extern crate calx_grid;
extern crate calx_color;
#[macro_use]
extern crate calx_ecs;

use euclid::Vector2D;

mod ability;

mod command;
pub use command::{Command, CommandResult};

mod components;
pub use components::Icon;

mod field;
mod flags;

mod form;
pub use form::{Form, FORMS};

mod fov;

mod item;
pub use item::{Slot, ItemType};

mod location;
pub use location::{Location, Portal};

mod location_set;
// TODO: Make private, trigger mapgen internally using higher-level API
pub mod mapgen;

mod mapfile;
pub use mapfile::{save_prefab, load_prefab};

mod mutate;
pub use mutate::Mutate;

mod query;
pub use query::Query;

mod spatial;
mod stats;

mod terraform;
pub use terraform::{Terraform, TerrainQuery};

pub mod terrain;
pub use terrain::Terrain;

mod world;
pub use world::{Ecs, World};

/// Standard Prefab type, terrain type and spawn name list.
pub type Prefab = calx_grid::Prefab<(Terrain, Vec<String>)>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FovStatus {
    Seen,
    Remembered,
}

pub type Rng = calx_alg::EncodeRng<rand::XorShiftRng>;

/// Return whether the given chart point is on the currently visible screen.
///
/// It is assumed that the chart point 0, 0 is at the center of the screen.
///
/// Since various bits of game logic are tied to the screen boundaries, the screen size is fixed as
/// a constant.
pub fn on_screen(chart_pos: Vector2D<i32>) -> bool {
    const W: i32 = 39;
    const H: i32 = 22;
    let (x, y) = (chart_pos.x, chart_pos.y);

    x <= y + (W + 1) / 2 // east
        && x >= y - (W - 1) / 2 // west
        && x >= -H - y // north
        && x <= H - 1 - y // south
}

/// List of all points for which `on_screen` is true.
pub fn onscreen_locations() -> &'static Vec<Vector2D<i32>> {
    lazy_static! {
        static ref ONSCREEN_LOCATIONS: Vec<Vector2D<i32>> = {
            let mut m = Vec::new();

            // XXX: Hardcoded limits, tied to W and H in on-screen but expressed differently here.
            for y in -21..22 {
                for x in -21..21 {
                    let point = Vector2D::new(x, y);
                    if on_screen(point) {
                        m.push(point);
                    }
                }
            }
            m
        };
    }

    &*ONSCREEN_LOCATIONS
}

/// The combat formula.
///
/// Given a deciban roll and the relevant stats, determine amount of damage dealt.
pub fn attack_damage(
    roll: f32,
    attack: i32,
    weapon_power: i32,
    target_defense: i32,
    target_armor: i32,
) -> i32 {
    const MAX_DAMAGE_MULTIPLIER: f32 = 4.0;

    let roll = roll + (attack - target_defense) as f32;
    (weapon_power as f32 *
         calx_alg::clamp(
            0.0,
            MAX_DAMAGE_MULTIPLIER,
            (roll - 2.0) * 0.05 - target_armor as f32 * 0.1,
        )) as i32
}

pub mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error) #[cfg(unix)];
            TomlDecode(::toml::de::Error);
        }
    }
}
