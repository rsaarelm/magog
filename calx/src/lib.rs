mod alg_misc;
mod colors;
pub mod ease;
mod fov;
mod hex;
mod hex_fov;
mod incremental;
mod legend_builder;
mod parser;
mod prefab;
mod rng;
mod search;
mod space;
mod system;
mod text;
mod timing;

pub use crate::alg_misc::{
    bounding_rect, clamp, compact_bits_by_2, lerp, retry_gen, spread_bits_by_2, Deciban,
    GenericError, LerpPath, Noise, WeightedChoice,
};
pub use crate::colors::{
    color, scolor, term_color, to_linear, to_srgb, BaseTermColor, PseudoTermColor, Rgba, SRgba,
    TermColor, Xterm256Color, NAMED_COLORS,
};
pub use crate::fov::{Fov, FovValue, PolarPoint};
pub use crate::hex::{hex_disc, hex_neighbors, taxicab_neighbors, Dir12, Dir6, HexDisc, HexGeom};
pub use crate::hex_fov::{AddFakeIsometricCorners, HexFov, HexFovIter, HexPolarPoint};
pub use crate::incremental::{History, Incremental, IncrementalState};
pub use crate::legend_builder::LegendBuilder;
pub use crate::prefab::{
    DenseTextMap, FromPrefab, IntoPrefab, MinimapSpace, PrefabError, ProjectedImage, TextSpace,
};
pub use crate::rng::{seeded_rng, RandomPermutation, RngExt};
pub use crate::search::{astar_path, Dijkstra, GridNode};
pub use crate::space::{CellSpace, CellVector, Space, Transformation};
pub use crate::system::{app_data_path, save_screenshot, TimeLogItem};
pub use crate::text::{split_line, templatize};
pub use crate::timing::{cycle_anim, single_anim, spike, TimestepLoop};
