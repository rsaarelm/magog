//! Set up resource content for game.

use vitral;
use calx_color::color::*;
use calx_resource::ResourceStore;
use world::BrushBuilder;

pub fn brushes<V: Copy + Eq>(builder: &mut vitral::Builder<V>) {
    BrushBuilder::new(builder)
        .file("content/assets/floors.png")
        ////
        .tile(0, 0)
        .brush("blank_floor")
        ////
        .color(DARKGREEN)
        .tile(32, 0)
        .brush("grass")
        ////
        .color(DARKGREEN)
        .tile(64, 0)
        .brush("grass2")
        ////
        .color(SLATEGRAY)
        .tile(32, 0)
        .brush("ground")
        ////
        .color(ROYALBLUE)
        .tile(96, 0)
        .brush("water")
        ////
        .file("content/assets/portals.png")
        ////
        .color(LIGHTCYAN)
        .tile(0, 0)
        .frame()
        .tile(32, 0)
        .frame()
        .tile(64, 0)
        .frame()
        .tile(96, 0)
        .frame()
        .tile(128, 0)
        .frame()
        .tile(160, 0)
        .frame()
        .tile(192, 0)
        .frame()
        .tile(224, 0)
        .frame()
        .tile(256, 0)
        .frame()
        .tile(288, 0)
        .frame()
        .tile(320, 0)
        .frame()
        .tile(352, 0)
        .frame()
        .tile(384, 0)
        .brush("gate")
        ////
        .file("content/assets/props.png")
        ////
        .color(RED)
        .splat(32, 0, 32, 32)
        .brush("cursor")
        ////
        .color(SADDLEBROWN)
        .tile(160, 64)
        .color(GREEN)
        .tile(192, 64)
        .brush("tree")
        ////
        .file("content/assets/walls.png")
        ////
        .color(LIGHTSLATEGRAY)
        .wall(0, 0, 32, 0)
        .brush("wall")
        ////
        .file("content/assets/blobs.png")
        ////
        .color(DARKGOLDENROD)
        .blob(0, 0, 0, 32, 0, 64)
        .brush("rock");
}

pub fn terrain() {
    use world::terrain::{Form, Kind, Tile};

    // Void, terrain 0 is special.
    Tile::insert_resource(0, Tile::new("blank_floor", Kind::Block, Form::Void));
    // "Level exit", a visible portal tile.
    Tile::insert_resource(1, Tile::new("gate", Kind::Ground, Form::Gate));
    Tile::insert_resource(2, Tile::new("ground", Kind::Ground, Form::Floor));
    Tile::insert_resource(3, Tile::new("grass", Kind::Ground, Form::Floor));
    Tile::insert_resource(4, Tile::new("water", Kind::Water, Form::Floor));
    Tile::insert_resource(5, Tile::new("tree", Kind::Block, Form::Prop));
    Tile::insert_resource(6, Tile::new("wall", Kind::Block, Form::Wall));
    Tile::insert_resource(7, Tile::new("rock", Kind::Block, Form::Blob));
}
