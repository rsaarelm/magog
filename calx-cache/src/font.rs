use std::collections::HashMap;
use image::{self, GenericImage};
use img::{tilesheet_bounds, subimage};
use ImageStore;

#[derive(Copy, Clone)]
pub struct Glyph {
    pub image_idx: usize,
    pub width: f32,
}

#[derive(Clone)]
pub struct Font {
    glyphs: HashMap<char, Glyph>,
}

impl Font {
    pub fn new<I, P, S>(tilesheet: &mut I, chars: &str, store: &mut S) -> Font
        where S: ImageStore,
              I: image::GenericImage<Pixel = P> + 'static,
              P: image::Pixel<Subpixel = u8> + PartialEq + 'static
    {
        let mut glyphs = HashMap::new();

        let bounds = tilesheet_bounds(tilesheet);
        for (ch, rect) in chars.chars().zip(bounds.iter()) {
            let sub = subimage(tilesheet, rect);
            let width = sub.width() as f32;
            let idx = store.add_image([0, sub.height() as i32], &sub);
            glyphs.insert(ch,
                          Glyph {
                              image_idx: idx,
                              width: width,
                          });
        }

        Font { glyphs: glyphs }
    }
}
