use std::io::{Writer, IoResult};
use util::{Rgb, V2};
use canvas::{Canvas, FONT_W, FONT_H};

/// Writing text to a graphical context.
pub trait Fonter<'a, W: Writer> {
    fn text_writer(&'a mut self, origin: V2<i32>, z: f32, color: Rgb) -> W;
}

impl<'a> Fonter<'a, CanvasWriter<'a>> for Canvas {
    fn text_writer(&'a mut self, origin: V2<i32>, z: f32, color: Rgb) -> CanvasWriter<'a> {
        CanvasWriter {
            canvas: self,
            origin: origin,
            cursor_pos: origin,
            color: color,
            z: z,
            border: None,
        }
    }
}

pub struct CanvasWriter<'a> {
    canvas: &'a mut Canvas,
    origin: V2<i32>,
    cursor_pos: V2<i32>,
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
        static BORDER: [V2<i32>; 8] =
            [V2(-1, -1), V2( 0, -1), V2( 1, -1),
             V2(-1,  0),             V2( 1,  0),
             V2(-1,  1), V2( 0,  1), V2( 1,  1)];
        if let Some(img) = self.canvas.font_image(c) {
            if let Some(b) = self.border {
                // Put the border a tiny bit further in the z-buffer so it
                // won't clobber the text on the same layer.
                let border_z = self.z + 0.00001;
                for &d in BORDER.iter() {
                    self.canvas.draw_image(self.cursor_pos + d, border_z, img, &b);
                }
            }
            self.canvas.draw_image(self.cursor_pos, self.z, img, &self.color);
        }
    }
}

impl<'a> Writer for CanvasWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        // TODO: Support multibyte stuff.
        for &b in buf.iter() {
            let c = b as char;
            if c == '\n' {
                self.cursor_pos = V2(self.origin.0, self.cursor_pos.1 + FONT_H as i32);
            } else {
                self.draw_char(c);
                self.cursor_pos.0 += FONT_W as i32;
            }
        }
        Ok(())
    }
}
