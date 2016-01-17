extern crate calx;
extern crate content;
extern crate world;

use calx::backend::{Key, Event};
use calx::backend::{WindowBuilder, Canvas, CanvasBuilder};
use content::TerrainType;
use world::Location;

pub fn main() {
    let window = WindowBuilder::new()
                     .set_title("Mapedit")
                     .build();

    let mut builder = CanvasBuilder::new();
    let mut ctx = builder.build(window);

    loop {
        for event in ctx.events().into_iter() {
            match event {
                Event::Quit => {
                    return;
                }
                _ => (),
            }
        }
        ctx.end_frame();
    }
}

pub struct EditState {
    center: Location,
    brush: TerrainType,
}

impl EditState {
    pub fn new() -> EditState {
        EditState {
            center: Location::new(0, 0),
            brush: TerrainType::Rock,
        }
    }
}
