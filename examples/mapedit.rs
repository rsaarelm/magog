extern crate cgmath;
extern crate calx_color;
extern crate calx_grid;
extern crate calx_cache;
extern crate calx_window;
extern crate calx_wall;
extern crate content;
extern crate world;
extern crate render;

use cgmath::{Vector2};
use calx_color::color;
use calx_cache::{AtlasBuilder};
use calx_window::{Key, Event, MouseButton, WindowBuilder, Window};
use calx_grid::Kernel;
use calx_wall::{Wall, DrawUtil};
use content::TerrainType;
use content::Brush;
use world::{World, Location};
use world::query;
use render::{chart_to_screen, view_to_chart, cells_on_screen, render_terrain};
use render::{Angle, FLOOR_Z, BLOCK_Z};

/// State object for configuring world display.
struct DrawState {
    pub center: Location,
    pub cursor_loc: Option<Location>,
}

struct Context {
    pub window: Window,
    pub wall: Wall,
}

impl Context {
    pub fn new() -> Context {
        let window = WindowBuilder::new()
                         .set_title("Mapedit")
                         .build();

        let mut builder = AtlasBuilder::new();
        content::Brush::init(&mut builder);
        let wall = Wall::new(&window.display, builder);
        Context {
            window: window,
            wall: wall,
        }
    }
}

impl DrawState {
    pub fn new(center: Location) -> DrawState {
        DrawState {
            center: center,
            cursor_loc: None,
        }
    }

    pub fn cursor(mut self, cursor_loc: Location) -> DrawState {
        self.cursor_loc = Some(cursor_loc);
        self
    }

    fn draw(&self, ctx: &mut Context, w: &World) {
        for pt in cells_on_screen() {
            let screen_pos = chart_to_screen(pt);
            let loc = self.center + pt;

            let k = Kernel::new(|x, y| query::terrain(w, loc + [x, y]));
            render_terrain(&k, |img, angle, fore, back| {
                let z = match angle {
                    Angle::Up => FLOOR_Z,
                    _ => BLOCK_Z,
                };
                ctx.wall.draw_image(img, screen_pos, z, fore, back)
            });
        }

        if let Some(cursor_loc) = self.cursor_loc {
            self.draw_cursor(ctx, cursor_loc)
        }
    }

    fn draw_cursor(&self, ctx: &mut Context, cursor_loc: Location) {
        if let Some(pt) = self.center.v2_at(cursor_loc) {
            // Draw cursor
            let screen_pos = chart_to_screen(pt);
            ctx.wall.draw_image(Brush::CursorBottom.get(0),
                           screen_pos,
                           FLOOR_Z,
                           color::RED,
                           color::BLACK);
            ctx.wall.draw_image(Brush::CursorTop.get(0),
                           screen_pos,
                           BLOCK_Z,
                           color::RED,
                           color::BLACK);

        }
    }
}

pub fn main() {
    let mut ctx = Context::new();

    let mut state = EditState::new();
    let mut cursor_pos = [0, 0];

    loop {
        ctx.window.clear(0x7799DDFF);
        let cursor_loc = state.center + view_to_chart(Vector2::from(cursor_pos));

        DrawState::new(state.center)
            .cursor(cursor_loc)
            .draw(&mut ctx, &state.world);

        for event in ctx.window.events().into_iter() {
            match event {
                Event::Quit => return,
                Event::KeyPress(Key::Escape) => return,

                Event::KeyPress(Key::W) => {
                    state.center = state.center + [-1, -1]
                }
                Event::KeyPress(Key::S) => {
                    state.center = state.center + [1, 1]
                }
                Event::KeyPress(Key::A) => {
                    state.center = state.center + [-1, 1]
                }
                Event::KeyPress(Key::D) => {
                    state.center = state.center + [1, -1]
                }

                Event::MouseMove(pos) => {
                    state.paint(cursor_loc);

                    let size = ctx.window.size();
                    cursor_pos = [pos[0] as i32 - (size[0] / 2) as i32,
                                  pos[1] as i32 - (size[1] / 2) as i32];
                }

                Event::MousePress(MouseButton::Left) => {
                    state.paint_state = Some(PaintState::Draw);
                    state.paint(cursor_loc);
                }

                Event::MousePress(MouseButton::Right) => {
                    state.paint_state = Some(PaintState::Clear);
                    state.paint(cursor_loc);
                }

                Event::MouseRelease(_) => {
                    state.paint_state = None;
                }

                _ => (),
            }
        }
        ctx.window.display(&mut ctx.wall);
        ctx.window.end_frame();
    }
}

enum PaintState {
    Draw,
    Clear,
}

pub struct EditState {
    world: World,
    center: Location,
    brush: TerrainType,
    paint_state: Option<PaintState>,
}

impl EditState {
    pub fn new() -> EditState {
        EditState {
            world: World::new(None),
            center: Location::new(0, 0),
            brush: TerrainType::Rock,
            paint_state: None,
        }
    }

    pub fn paint(&mut self, cursor_loc: Location) {
        match self.paint_state {
            Some(PaintState::Draw) => {
                self.world.terrain.set(cursor_loc, self.brush)
            }
            Some(PaintState::Clear) => self.world.terrain.clear(cursor_loc),
            None => {}
        }
    }
}
