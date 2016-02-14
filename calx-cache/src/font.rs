use std::collections::HashMap;
use image::{self, GenericImage};
use img::{tilesheet_bounds, subimage};
use ImageStore;

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
    pub fn new<I, P, S>(tilesheet: &mut I,
                        chars: &str,
                        store: &mut S)
                        -> Font<H>
        where S: ImageStore<H>,
              I: image::GenericImage<Pixel = P> + 'static,
              P: image::Pixel<Subpixel = u8> + PartialEq + 'static
    {
        let mut glyphs = HashMap::new();

        let bounds = tilesheet_bounds(tilesheet);
        for (ch, rect) in chars.chars().zip(bounds.iter()) {
            let sub = subimage(tilesheet, rect);
            let width = sub.width() as f32;
            let h = store.add_image([0, sub.height() as i32], &sub);
            glyphs.insert(ch,
                          Glyph {
                              image: h,
                              width: width,
                          });
        }

        Font { glyphs: glyphs }
    }
}
