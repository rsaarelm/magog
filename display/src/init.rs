//! Set up resource content for game.

use glium;
use calx_color::color::*;
use calx_resource::ResourceStore;
use world::BrushBuilder;
use backend::Backend;

/// Init the static brush assets.
pub fn brushes(display: &glium::Display, backend: &mut Backend) {
    BrushBuilder::new()
        .file("display/assets/floors.png")
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
        .file("display/assets/portals.png")
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
        .file("display/assets/props.png")
        ////
        .color(RED)
        .tile(0, 0)
        .brush("cursor")
        .color(RED)
        .tile(32, 0)
        .brush("cursor_top")
        .color(ORANGE)
        .tile(0, 0)
        .brush("portal") // TODO: This thing is a temporary display helper for mapedit, maybe remove when can?
        ////
        .color(SADDLEBROWN)
        .tile(160, 64)
        .color(GREEN)
        .tile(192, 64)
        .brush("tree")
        ////
        .file("display/assets/walls.png")
        ////
        .color(LIGHTSLATEGRAY)
        .wall(0, 0, 32, 0)
        .brush("wall")
        ////
        .file("display/assets/blobs.png")
        ////
        .color(DARKGOLDENROD)
        .blob(0, 0, 0, 32, 0, 64)
        .brush("rock")
        ////
        .file("display/assets/mobs.png")
        .color(AZURE)
        .tile(0, 0)
        .bob()
        .brush("player")
        .color(GREEN)
        .tile(32, 0)
        .bob()
        .brush("snake")
        .color(OLIVE)
        .tile(64, 0)
        .bob()
        .brush("dreg")
        ////
        .finish(|img| backend.make_texture(&display, img));
}
