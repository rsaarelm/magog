use std::collections::HashMap;
use image::{self, GenericImage};
use img::{ImageStore, tilesheet_bounds, subimage};

#[derive(Copy, Clone)]
pub struct Glyph<H> {
    pub image: H,
    pub width: f32,
}

#[derive(Clone)]
pub struct Font<H> {
    glyphs: HashMap<char, Glyph<H>>,
}

impl<H> Font<H> {
    pub fn new<I, S>(tilesheet: &mut I, chars: &str, store: &mut S) -> Font<H>
        where S: ImageStore<H>,
              I: image::GenericImage + 'static,
              I::Pixel: PartialEq + 'static,
              <I::Pixel as image::Pixel>::Subpixel: 'static
    {
        let mut glyphs = HashMap::new();

        let bounds = tilesheet_bounds(tilesheet);
        for (ch, rect) in chars.chars().zip(bounds.iter()) {
            let sub = subimage(tilesheet, rect);
            let width = sub.width() as f32;
            let h = store.add_image([0, -(sub.height() as i32)], &sub);
            glyphs.insert(ch, Glyph { image: h, width: width });
        }

        Font { glyphs: glyphs }
    }
}
