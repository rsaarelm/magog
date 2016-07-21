extern crate euclid;
#[macro_use]
extern crate glium;
extern crate image;
extern crate vitral;
extern crate serde;
extern crate calx_color;
#[macro_use]
extern crate calx_resource;
extern crate world;

mod backend;

use calx_color::color::*;
pub use euclid::{Point2D, Rect, Size2D};
pub use glium::{DisplayBuild, glutin};
pub use backend::Backend;
pub use calx_resource::{Loadable, Resource, ResourceCache, ResourceStore};
use world::{Frame, BrushBuilder, Brush};


fn init_brushes<V: Copy + Eq>(builder: &mut vitral::Builder<V>) {
    BrushBuilder::new(builder)
        .file("content/assets/floors.png")

        .splat(0, 0, 32, 32).offset(16, 16).brush("blank_floor")

        .color(DARKGREEN).tile(32, 0).brush("grass")

        .color(DARKGREEN).tile(64, 0).brush("grass2")

        .color(SLATEGRAY).tile(32, 0).brush("ground")

        .color(ROYALBLUE).tile(96, 0).brush("water")

        .file("content/assets/props.png")

        .color(RED).splat(32, 0, 32, 32).brush("cursor")

        .color(SADDLEBROWN).tile(160, 64)
        .color(GREEN).tile(192, 64).brush("tree")

        .file("content/assets/walls.png")
        .color(LIGHTSLATEGRAY)
        .wall(0, 0, 32, 0).brush("wall")

        .file("content/assets/blocks.png")
        .color(DARKGOLDENROD)
        .block(0, 0, 0, 32, 0, 64).brush("rock")

        ;
}

fn draw_frame(context: &mut backend::Context, offset: Point2D<f32>, frame: &Frame) {
    for splat in frame.iter() {
        context.draw_image(splat.image, offset + splat.offset, splat.color);
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Kind {
    Ground,
    Block,
    Water,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Form {
    /// Single frame on floor layer
    Floor,
    /// Single frame on object layer
    Prop,
    /// Block-form on object layer
    Block,
    /// Wall-form on object layer
    Wall,
}

#[derive(Clone)]
pub struct Tile {
    pub brush: Resource<Brush>,
    pub kind: Kind,
    pub form: Form,
}

impl Tile {
    pub fn new(brush: &str, kind: Kind, form: Form) -> Tile {
        Tile {
            brush: Resource::new(brush.to_string()).unwrap(),
            kind: kind,
            form: form,
        }
    }
}

impl Loadable<u8> for Tile {}

impl_store!(TILE, u8, Tile);

fn init_terrain() {
    Tile::insert_resource(0, Tile::new("ground", Kind::Ground, Form::Floor));
    Tile::insert_resource(1, Tile::new("grass", Kind::Ground, Form::Floor));
    Tile::insert_resource(2, Tile::new("water", Kind::Water, Form::Floor));
    Tile::insert_resource(3, Tile::new("tree", Kind::Block, Form::Prop));
    Tile::insert_resource(4, Tile::new("wall", Kind::Block, Form::Wall));
    Tile::insert_resource(5, Tile::new("rock", Kind::Block, Form::Block));
}

pub fn main() {
    let display = glutin::WindowBuilder::new()
                      .build_glium()
                      .unwrap();

    let mut backend = Backend::new(&display);

    // Construct Vitral context.
    let mut context: backend::Context;
    let mut builder = vitral::Builder::new();

    init_brushes(&mut builder);
    init_terrain();

    context = builder.build(|img| backend.make_texture(&display, img));

    let font = context.default_font();

    loop {
        context.begin_frame();

        context.draw_text(font,
                          Point2D::new(4.0, 20.0),
                          [1.0, 1.0, 1.0, 1.0],
                          "Hello, world!");

        draw_frame(&mut context,
                   Point2D::new(50.0, 50.0),
                   &Brush::get_resource(&"tree".to_string()).unwrap()[0]);

        if !backend.update(&display, &mut context) {
            return;
        }
    }
}
