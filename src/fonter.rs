use std::io::{Writer, IoResult};
use super::Rgb;
use geom::{V2};
use canvas::{Context, FONT_W, FONT_H};

/// Writing text to a graphical context.
pub trait Fonter<'a, W: Writer> {
    fn text_writer(&'a mut self, origin: V2<int>, z: f32, color: Rgb) -> W;
}

impl<'a> Fonter<'a, CanvasWriter<'a>> for Context {
    fn text_writer(&'a mut self, origin: V2<int>, z: f32, color: Rgb) -> CanvasWriter<'a> {
        CanvasWriter {
            context: self,
            origin: origin,
            cursor_pos: origin,
            color: color,
            z: z,
            border: None,
        }
    }
}

pub struct CanvasWriter<'a> {
    context: &'a mut Context,
    origin: V2<int>,
    cursor_pos: V2<int>,
    /// Text color
    pub color: Rgb,
    /// Z drawing depth
    pub z: f32,
    /// Border color (if any)
    pub border: Option<Rgb>,
}

impl<'a> CanvasWriter<'a> {
    pub fn set_border(mut self, c: Rgb) -> CanvasWriter<'a> {
        self.border = Some(c);
        self
    }

    fn draw_char(&mut self, c: char) {
        static BORDER: [V2<int>, ..8] =
            [V2(-1, -1), V2( 0, -1), V2( 1, -1),
             V2(-1,  0),             V2( 1,  0),
             V2(-1,  1), V2( 0,  1), V2( 1,  1)];
        if let Some(img) = self.context.font_image(c) {
            if let Some(b) = self.border {
                // Put the border a tiny bit further in the z-buffer so it
                // won't clobber the text on the same layer.
                let border_z = self.z + 0.00001;
                for &d in BORDER.iter() {
                    self.context.draw_image(self.cursor_pos + d, border_z, img, &b);
                }
            }
            self.context.draw_image(self.cursor_pos, self.z, img, &self.color);
        }
    }
}

impl<'a> Writer for CanvasWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        // TODO: Support multibyte stuff.
        for &b in buf.iter() {
            let c = b as char;
            if c == '\n' {
                self.cursor_pos = V2(self.origin.0, self.cursor_pos.1 + FONT_H as int);
            } else {
                self.draw_char(c);
                self.cursor_pos.0 += FONT_W as int;
            }
        }
        Ok(())
    }
}
