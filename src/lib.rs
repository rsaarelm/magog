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

/// Unit tag for typed euclid structs.
///
/// `CellSpace` is the internal default space for game maps that spaces with other projections can
/// transform to and from usind `std::convert`.
///
/// When used for hex maps, `CellSpace` is still a regular euclidean space and does not use axis
/// staggering. Squares in `CellSpace` coordinates are distorted into lozenges when displayed as a
/// hex map. For a flat-top hex map, the convention is that the positive x-axis points to the
/// 4-o'clock (southeast) neighbor hex and the positive y-axis points to the 8-o'clock (southwest)
/// neighbor hex. For a pointy-top hex map, x-axis points to the 3-o'clock (east) neighbor hex and
/// y-axis points to the 7-o'clock (southwest) neighbor hex.
pub struct CellSpace;
pub type CellVector = euclid::TypedVector2D<i32, CellSpace>;

pub use alg_misc::{clamp, noise, lerp, sorted_pair, spread_bits_by_2, compact_bits_by_2,
                   retry_gen, Deciban, WeightedChoice};
pub use colors::{Rgba, SRgba, to_linear, to_srgb, NAMED_COLORS, scolor, color};
pub use fov::{Fov, FovValue, PolarPoint};
pub use hex::{HexGeom, hex_neighbors, HexNeighbor, hex_disc, HexDisc, Dir6, Dir12};
pub use hex_fov::{HexFov, HexPolarPoint, HexFovIter, AddFakeIsometricCorners};
pub use prefab::{Prefab, IntoPrefab};
pub use rng::{RngExt, EncodeRng, RandomPermutation};
pub use search::{GridNode, Dijkstra, astar_path_with};
pub use system::{app_data_path, save_screenshot, TimeLogItem};
pub use text::{split_line, templatize};
pub use timing::{cycle_anim, spike, single_anim, Ticker, TimePerFrame, AverageDuration};
