extern crate euclid;
extern crate image;
extern crate num;
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
mod legend_builder;
mod parser;
mod prefab;
mod rng;
mod search;
mod space;
mod system;
mod text;
mod timing;

pub use alg_misc::{bounding_rect, clamp, lerp, noise, retry_gen, sorted_pair, Deciban,
                   WeightedChoice, compact_bits_by_2, spread_bits_by_2};
pub use colors::{color, scolor, to_linear, to_srgb, Rgba, SRgba, NAMED_COLORS};
pub use fov::{Fov, FovValue, PolarPoint};
pub use hex::{hex_disc, hex_neighbors, Dir12, Dir6, HexDisc, HexGeom, HexNeighbor};
pub use hex_fov::{AddFakeIsometricCorners, HexFov, HexFovIter, HexPolarPoint};
pub use legend_builder::LegendBuilder;
pub use prefab::{IntoPrefab, FromPrefab, MinimapSpace, PrefabError, ProjectedImage, TextSpace};
pub use rng::{EncodeRng, IndependentSample, RandomPermutation, RngExt, SampleIterator};
pub use search::{astar_grid, astar_path, Dijkstra, GridNode};
pub use space::{CellSpace, CellVector, Space, Transformation};
pub use system::{app_data_path, save_screenshot, TimeLogItem};
pub use text::{split_line, templatize};
pub use timing::{cycle_anim, single_anim, spike, AverageDuration, Ticker, TimePerFrame};
