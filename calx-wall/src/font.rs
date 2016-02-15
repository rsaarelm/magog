use std::collections::HashMap;
use cgmath::{Vector2, vec2};
use image::{self, GenericImage};
use calx_cache::{ImageStore, tilesheet_bounds, subimage};
use calx_layout::{Rect, Anchor};
use calx_color::{Rgba, color};
use calx_alg::wrap_lines;
use wall::Wall;
use draw_util::DrawUtil;

#[derive(Copy, Clone)]
pub struct Glyph {
    pub image_idx: usize,
    pub width: f32,
}

#[derive(Clone)]
pub struct Font {
    height: f32,
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
        assert!(chars.len() >= 1);
        assert!(bounds.len() >= chars.len());

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

        Font {
            // XXX: Assume that all font tiles have the correct height.
            height: bounds[0].size[1] as f32,
            glyphs: glyphs,
        }
    }

    pub fn char_width(&self, c: char) -> f32 {
        if let Some(ref glyph) = self.glyphs.get(&c) {
            glyph.width
        } else {
            self.height / 2.0
        }
    }

    pub fn get<'a>(&'a self, c: char) -> Option<&'a Glyph> {
        self.glyphs.get(&c)
    }
}

/// Text line alignment.
pub enum Align {
    Left,
    Right,
    Center,
}

/// Structure for rendering text to a canvas.
pub struct Fonter<'a, 'b> {
    wall: &'a mut Wall,
    font: &'b Font,
    anchor: Anchor,
    align: Align,
    color: Rgba,
    z: f32,
    max_lines: Option<usize>,
    border: Option<Rgba>,
    max_width: Option<f32>,
    /// (Text, width) pairs.
    lines: Vec<(String, f32)>,
    longest_line_width: f32,
}

impl<'a, 'b> Fonter<'a, 'b> {
    pub fn new(wall: &'a mut Wall, font: &'b Font) -> Fonter<'a, 'b> {
        Fonter {
            wall: wall,
            font: font,
            anchor: Anchor::TopLeft,
            align: Align::Left,
            color: color::WHITE,
            z: 0.1,
            max_lines: None,
            border: None,
            max_width: None,
            lines: vec![(String::new(), 0.0)],
            longest_line_width: 0.0,
        }
    }

    /// Set the point of the text box which draw offset will anchor to.
    pub fn anchor(mut self, anchor: Anchor) -> Fonter<'a, 'b> {
        self.anchor = anchor;
        self
    }

    /// Set the text alignment
    pub fn align(mut self, align: Align) -> Fonter<'a, 'b> {
        self.align = align;
        self
    }

    /// Set text color. The default color is white.
    pub fn color<C: Into<Rgba>>(mut self, color: C) -> Fonter<'a, 'b> {
        self.color = color.into();
        self
    }

    /// Set border color. Before this is set, the drawn text will not be drawn
    /// with a border.
    pub fn border<C: Into<Rgba>>(mut self, color: C) -> Fonter<'a, 'b> {
        self.border = Some(color.into());
        self
    }

    /// Set the z-layer to draw in.
    pub fn layer(mut self, z: f32) -> Fonter<'a, 'b> {
        self.z = z;
        self
    }

    /// Set the maximum width of the text area in pixels
    pub fn width(mut self, w: f32) -> Fonter<'a, 'b> {
        self.max_width = Some(w);
        self
    }

    /// Set the maximum number of lines to draw (lines of text before this are
    /// dropped).
    pub fn max_lines(mut self, max_lines: usize) -> Fonter<'a, 'b> {
        self.max_lines = Some(max_lines);
        self.cull_lines();
        self
    }

    /// Append to the fonter text.
    pub fn text(mut self, txt: String) -> Fonter<'a, 'b> {
        assert!(self.lines.len() > 0);
        // The last line can be added to, snip it off.
        let mut new_txt = format!("{}{}",
                                  self.lines[self.lines.len() - 1].0,
                                  txt);
        let new_len = self.lines.len() - 1;
        self.lines.truncate(new_len);
        if let Some(w) = self.max_width {
            new_txt = wrap_lines(&new_txt[..], &|c| self.font.char_width(c), w);
        }

        let new_lines: Vec<(String, f32)> = new_txt.split('\n')
                                                   .map(|s| {
                                                       (s.to_string(),
                                                        self.str_width(s))
                                                   })
                                                   .collect();
        self.lines.extend(new_lines.into_iter());
        assert!(self.lines.len() > 0);

        self.cull_lines();
        self
    }

    fn cull_lines(&mut self) {
        if let Some(n) = self.max_lines {
            // XXX: Removing items one-by-one from a Vec is iffy
            // performance, but the usual use case here should have only
            // one or two lines beyond the limit. The unstable
            // Vec::split_off method would be a more generally effective
            // approach here.
            while self.lines.len() > n {
                self.lines.remove(0);
            }
        }
        self.set_longest_width();
    }

    fn set_longest_width(&mut self) {
        self.longest_line_width = self.lines
                                      .iter()
                                      .map(|x| x.1)
                                      .fold(0.0, |a, w| {
                                          if w > a {
                                              w
                                          } else {
                                              a
                                          }
                                      });
    }

    fn draw_char<V, C, D>(&mut self,
                          c: char,
                          offset: V,
                          z: f32,
                          color: C,
                          border: Option<D>)
        where V: Into<[f32; 2]> + Copy,
              C: Into<Rgba> + Copy,
              D: Into<Rgba> + Copy
    {
        if let Some(ref glyph) = self.font.get(c) {
            if let Some(border) = border {
                use calx_layout::Anchor::*;
                let border_pts = Rect::new(Vector2::from(offset.into()) +
                                           vec2(-1.0, -1.0),
                                           Vector2::from(offset.into()) +
                                           vec2(1.0, 1.0));
                for pt in [TopLeft,
                           Top,
                           TopRight,
                           Left,
                           Right,
                           BottomLeft,
                           Bottom,
                           BottomRight]
                              .into_iter() {
                    // Put the border a tiny bit further in the z-buffer so it
                    // won't clobber the text on the same layer.

                    self.wall.draw_image(glyph.image_idx,
                                         border_pts.point(*pt),
                                         z + 0.00001,
                                         border,
                                         color::BLACK);
                }
            }
            self.wall
                .draw_image(glyph.image_idx, offset, z, color, color::BLACK);
        }
    }

    /// Render the fonter text to canvas.
    pub fn draw<V: Into<[f32; 2]>>(&mut self, offset: V) {
        let offset = Vector2::from(offset.into());
        let anchor_points =
            Rect::new([0.0, 0.0],
                      [self.longest_line_width,
                       (self.lines.len() * self.font.height as usize) as f32]);
        let offset = offset - Vector2::from(anchor_points.point(self.anchor));
        // TODO Anchoring
        let (z, color, border) = (self.z, self.color, self.border);
        for (row, s) in self.lines.clone().iter().enumerate() {
            let y = offset[1] + self.font.height as f32 +
                    row as f32 * self.font.height as f32;
            let line_width = s.1;
            let mut x = offset[0] +
                        match self.align {
                Align::Left => 0.0,
                Align::Right => (self.longest_line_width - line_width),
                Align::Center => (self.longest_line_width - line_width) / 2.0,
            };
            for c in s.0.chars() {
                self.draw_char(c, [x, y], z, color, border);
                x += self.font.char_width(c);
            }
        }
    }


    fn str_width(&self, s: &str) -> f32 {
        s.chars().fold(0.0, |a, c| a + self.font.char_width(c))
    }
}
