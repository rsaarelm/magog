//! Set up resource content for game.

use std::collections::HashMap;
use std::iter::FromIterator;
use glium;
use image::{self, GenericImage};
use euclid::Point2D;
use vitral_atlas;
use vitral;
use calx_color::color::*;
use calx_resource::ResourceStore;
use world::BrushBuilder;
use backend::{Backend, Font};

impl_store!(FONT, String, Font);

fn to_image_buffer<I>(image: &I) -> vitral::ImageBuffer
    where I: image::GenericImage<Pixel = image::Rgba<u8>>
{
    vitral::ImageBuffer::from_iter(image.width(),
                                   image.height(),
                                   &mut image.pixels().map(|p| unsafe {
                                       ::std::mem::transmute::<image::Rgba<u8>, u32>(p.2)
                                   }))
}

pub fn font(display: &glium::Display, backend: &mut Backend) {
    use tilesheet::tilesheet_bounds;

    const PATH: &'static str = "display/assets/font.png";

    let mut font_sheet = image::open(PATH).expect(&format!("Font sheet '{}' not found", PATH));

    // Use the tilesheet machinery to extract individual glyph boundaries from the special font
    // tilesheet.
    let bounds = tilesheet_bounds(&font_sheet);

    // Construct Vitral ImageBuffers of the glyphs.
    let glyphs: Vec<vitral::ImageBuffer> = bounds.iter()
                                                 .map(|rect| {
                                                     to_image_buffer(&font_sheet.sub_image(
                rect.origin.x as u32, rect.origin.y as u32,
                rect.size.width as u32, rect.size.height as u32))
                                                 })
                                                 .collect();

    assert!(glyphs.len() == 96, "Unexpected number of font glyphs");
    let font_height = glyphs[0].size.height;

    // Build the font atlas and map the items to CharData.
    let items: Vec<vitral::CharData<usize>> = vitral_atlas::build(&glyphs, 2048, |img| {
                                                  backend.make_texture(display, img)
                                              })
                                                  .expect("Atlas construction failed")
                                                  .into_iter()
                                                  .map(|image| {
                                                      let advance = image.size.width as f32;
                                                      vitral::CharData {
                                                          image: image,
                                                          draw_offset:
                                                              Point2D::new(0.0, font_height as f32),
                                                          advance: advance,
                                                      }
                                                  })
                                                  .collect();

    let font = Font(vitral::FontData {
        chars: HashMap::from_iter((32u8..128).map(|c| c as char).zip(items.into_iter())),
        height: font_height as f32,
    });

    Font::insert_resource("default".to_string(), font);
}

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
        .color(SLATEGRAY)
        .tile(32, 0)
        .brush("corridor")
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
