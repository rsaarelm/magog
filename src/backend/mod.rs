/*!
Window-wrangling, polygon-pushing and input-grabbing

*/

pub use backend::canvas::{CanvasBuilder, Canvas};
pub use backend::canvas::{Image};
pub use backend::canvas_util::{CanvasUtil};
pub use backend::fonter::{Fonter, Align};

mod canvas;
mod canvas_util;
mod fonter;
pub mod mesh;


/// UI Widget static identifier, unique for a specific site in source code.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WidgetId {
    filename: &'static str,
    line: u32,
    column: u32,
}

pub trait RenderTarget {
    fn add_mesh(&mut self, vertices: Vec<mesh::Vertex>, faces: Vec<[u16; 3]>);
}

impl WidgetId {
    pub fn new(filename: &'static str, line: u32, column: u32) -> WidgetId {
        WidgetId {
            filename: filename,
            line: line,
            column: column,
        }
    }

    pub fn dummy() -> WidgetId {
        WidgetId {
            filename: "n/a",
            line: 666666,
            column: 666666,
        }
    }
}

#[macro_export]
/// Generate a static identifier for the current source code position. Used
/// with imgui API.
macro_rules! widget_id {
    () => {
        ::calx::backend::WidgetId::new(concat!(module_path!(), "/", file!()), line!(), column!())
    }
}
