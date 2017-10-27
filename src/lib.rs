extern crate euclid;
extern crate num;
extern crate image;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tempdir;
extern crate time;
extern crate vec_map;

mod alg_misc;
mod colors;
pub mod ease;
mod fov;
mod hex;
mod hex_fov;
mod parser;
mod prefab;
mod rng;
mod search;
mod system;
mod text;
mod timing;

pub use alg_misc::{clamp, noise, lerp, sorted_pair, spread_bits_by_2, compact_bits_by_2,
                   retry_gen, Deciban, WeightedChoice};
pub use colors::{Rgba, SRgba, to_linear, to_srgb, NAMED_COLORS, scolor, color};
pub use fov::{Fov, FovValue, PolarPoint};
pub use hex::{HexGeom, hex_neighbors, HexNeighbor, hex_disc, HexDisc, Dir6, Dir12};
pub use hex_fov::{HexFov, HexPolarPoint, HexFovIter, AddFakeIsometricCorners};
pub use prefab::{Prefab, HexmapDisplay, LegendBuilder};
pub use rng::{RngExt, EncodeRng, RandomPermutation};
pub use search::{GridNode, Dijkstra, astar_path_with};
pub use system::{app_data_path, save_screenshot, TimeLogItem};
pub use text::{split_line, templatize};
pub use timing::{cycle_anim, spike, single_anim, Ticker, TimePerFrame, AverageDuration};
