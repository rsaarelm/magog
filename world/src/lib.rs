extern crate image;
extern crate euclid;
extern crate vitral;
extern crate calx_color;
#[macro_use]
extern crate calx_ecs;
#[macro_use]
extern crate calx_resource;

mod brush;
pub mod terrain;

pub use brush::{Brush, BrushBuilder, Color, Frame, ImageRef, Splat};
