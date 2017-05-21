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

use euclid::Point2D;

mod ability;

mod command;
pub use command::Command;

mod components;
pub use components::Icon;

mod field;
mod flags;

mod form;
pub use form::{Form, FORMS};

mod fov;
mod item;

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
pub fn on_screen(chart_pos: Point2D<i32>) -> bool {
    const W: i32 = 39;
    const H: i32 = 22;
    let (x, y) = (chart_pos.x, chart_pos.y);

    x <= y + (W + 1) / 2 // east
        && x >= y - (W - 1) / 2 // west
        && x >= -H - y // north
        && x <= H - 1 - y // south
}

/// List of all points for which `on_screen` is true.
pub fn onscreen_locations() -> &'static Vec<Point2D<i32>> {
    lazy_static! {
        static ref ONSCREEN_LOCATIONS: Vec<Point2D<i32>> = {
            let mut m = Vec::new();

            // XXX: Hardcoded limits, tied to W and H in on-screen but expressed differently here.
            for y in -21..22 {
                for x in -21..21 {
                    let point = Point2D::new(x, y);
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

pub mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error) #[cfg(unix)];
            TomlDecode(::toml::de::Error);
        }
    }
}
