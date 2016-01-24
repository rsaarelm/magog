extern crate calx;
extern crate content;
extern crate world;
extern crate render;

use calx::backend::{Key, Event};
use calx::{Kernel, V2};
use calx::backend::{WindowBuilder, Canvas, CanvasUtil, CanvasBuilder};
use content::TerrainType;
use world::{World, Location};
use world::query;
use render::{chart_to_screen, cells_on_screen, render_terrain};
use render::{Angle, FLOOR_Z, BLOCK_Z};

fn draw_world(w: &World, ctx: &mut Canvas, center: Location) {
    for pt in cells_on_screen() {
        let screen_pos = chart_to_screen(pt);
        let loc = center + pt;

        let k = Kernel::new(|loc| query::terrain(w, loc), loc);
        render_terrain(&k, |img, angle, fore, back| {
            let z = match angle {
                Angle::Up => FLOOR_Z,
                _ => BLOCK_Z,
            };
            ctx.draw_image(img, screen_pos, z, fore, back)
        });
    }
}

pub fn main() {
    let window = WindowBuilder::new()
                     .set_title("Mapedit")
                     .build();

    let mut builder = CanvasBuilder::new();
    content::Brush::init(&mut builder);
    let mut ctx = builder.build(window);

    let mut state = EditState::new();

    loop {
        draw_world(&state.world, &mut ctx, state.center);

        for event in ctx.events().into_iter() {
            match event {
                Event::Quit => return,
                Event::KeyPress(Key::Escape) => return,

                Event::KeyPress(Key::W) => {
                    state.center = state.center + V2(-1, -1)
                }
                Event::KeyPress(Key::S) => {
                    state.center = state.center + V2(1, 1)
                }
                Event::KeyPress(Key::A) => {
                    state.center = state.center + V2(-1, 1)
                }
                Event::KeyPress(Key::D) => {
                    state.center = state.center + V2(1, -1)
                }

                _ => (),
            }
        }
        ctx.end_frame();
    }
}

pub struct EditState {
    world: World,
    center: Location,
    brush: TerrainType,
}

impl EditState {
    pub fn new() -> EditState {
        EditState {
            world: World::new(None),
            center: Location::new(0, 0),
            brush: TerrainType::Rock,
        }
    }
}
