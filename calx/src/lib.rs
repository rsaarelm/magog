mod alg_misc;
mod cell;
mod colors;
mod deprecated_space;
pub mod ease;
mod hex;
mod hex_fov;
mod incremental;
mod legend_builder;
mod parser;
mod prefab;
mod rng;
mod search;
pub mod stego;
mod system;
mod text;
pub mod tiled;

pub use alg_misc::{
    bounding_rect, compact_bits_by_2, lerp, retry_gen, spread_bits_by_2, Clamp,
    Deciban, GenericError, LerpPath, Noise, WeightedChoice,
};
pub use cell::{CellSpace, CellVector, Fov, FovValue, PolarPoint};
pub use colors::{
    term_color, BaseTermColor, PseudoTermColor, TermColor, Xterm256Color,
};
pub use deprecated_space::{DeprecatedSpace, Transformation};
pub use hex::{
    hex_disc, hex_neighbors, taxicab_neighbors, Dir12, Dir6, HexDisc, HexGeom,
};
pub use hex_fov::{AddFakeIsometricCorners, HexFov, HexFovIter, HexPolarPoint};
pub use incremental::{History, Incremental, IncrementalState};
pub use legend_builder::LegendBuilder;
pub use prefab::{
    DenseTextMap, FromPrefab, IntoPrefab, MinimapSpace, PrefabError,
    ProjectedImage, TextSpace,
};
pub use rng::{seeded_rng, RandomPermutation, RngExt};
pub use search::{astar_path, Dijkstra, GridNode};
pub use system::{app_data_path, save_screenshot, TimeLogItem};
pub use text::{split_line, templatize};
