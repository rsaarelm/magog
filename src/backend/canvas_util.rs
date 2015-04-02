use super::canvas::{Canvas, Image, FONT_W};
use super::{WidgetId};
use ::{V2, Rect, Color, Rgba, color};
use ::Anchor::*;

/// Helper methods for canvas context that do not depend on the underlying
/// implementation details.
pub trait CanvasUtil {
    /// Draw a thick solid line on the canvas.
    fn draw_line<C: Color+Copy>(&mut self, width: u32, p1: V2<f32>, p2: V2<f32>, layer: f32, color: &C);
    /// Get the size of an atlas image.
    fn image_dim(&self, img: Image) -> V2<u32>;

    /// Draw a stored image on the canvas.
    fn draw_image<C: Color+Copy, D: Color+Copy>(&mut self, img: Image,
        offset: V2<f32>, z: f32, color: &C, back_color: &D);

    /// Draw a filled rectangle
    fn fill_rect<C: Color+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: &C);

    // TODO: More specs
    fn button(&mut self, id: WidgetId, pos: V2<f32>, z: f32) -> bool;

    fn draw_char<C: Color+Copy, D: Color+Copy>(&mut self, c: char, offset: V2<f32>, z: f32, color: &C, border: Option<&D>);

    fn char_width(&self, c: char) -> f32;
}

impl CanvasUtil for Canvas {
    fn draw_line<C: Color+Copy>(&mut self, width: u32, p1: V2<f32>, p2: V2<f32>, layer: f32, color: &C) {
        let tex = self.solid_tex_coord();
        let v1 = p2 - p1;
        let v2 = V2(-v1.1, v1.0);

        let scalar = width as f32 / 2.0 * 1.0 / v2.dot(v2).sqrt();
        let v2 = v2 * scalar;

        let ind0 = self.num_vertices();
        self.push_vertex(p1 + v2, layer, tex, color, &color::BLACK);
        self.push_vertex(p1 - v2, layer, tex, color, &color::BLACK);
        self.push_vertex(p1 - v2 + v1, layer, tex, color, &color::BLACK);
        self.push_vertex(p1 + v2 + v1, layer, tex, color, &color::BLACK);
        self.push_triangle(ind0, ind0 + 1, ind0 + 2);
        self.push_triangle(ind0, ind0 + 2, ind0 + 3);
    }

    fn image_dim(&self, img: Image) -> V2<u32> {
        self.image_data(img).pos.1.map(|x| x as u32)
    }

    fn draw_image<C: Color+Copy, D: Color+Copy>(&mut self, img: Image,
        offset: V2<f32>, z: f32, color: &C, back_color: &D) {
        // Use round numbers, fractions seem to cause artifacts to pixels.
        let offset = offset.map(|x| x.floor());
        let mut pos;
        let mut tex;
        {
            let data = self.image_data(img);
            pos = data.pos + offset;
            tex = data.tex;
        }

        let ind0 = self.num_vertices();

        self.push_vertex(pos.point(TopLeft), z, tex.point(TopLeft), color, back_color);
        self.push_vertex(pos.point(TopRight), z, tex.point(TopRight), color, back_color);
        self.push_vertex(pos.point(BottomRight), z, tex.point(BottomRight), color, back_color);
        self.push_vertex(pos.point(BottomLeft), z, tex.point(BottomLeft), color, back_color);

        self.push_triangle(ind0, ind0 + 1, ind0 + 2);
        self.push_triangle(ind0, ind0 + 2, ind0 + 3);
    }

    fn fill_rect<C: Color+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: &C) {
        let tex = self.solid_tex_coord();
        let ind0 = self.num_vertices();

        self.push_vertex(rect.point(TopLeft), z, tex, color, &color::BLACK);
        self.push_vertex(rect.point(TopRight), z, tex, color, &color::BLACK);
        self.push_vertex(rect.point(BottomRight), z, tex, color, &color::BLACK);
        self.push_vertex(rect.point(BottomLeft), z, tex, color, &color::BLACK);

        self.push_triangle(ind0, ind0 + 1, ind0 + 2);
        self.push_triangle(ind0, ind0 + 2, ind0 + 3);
    }

    fn button(&mut self, id: WidgetId, pos: V2<f32>, z: f32) -> bool {
        // TODO: Button visual style! Closures?
        let area = Rect(pos, V2(64.0, 16.0));
        let mut color = Rgba::parse("green");
        if area.contains(&self.mouse_pos) {
            self.hot_widget = Some(id);
            if self.active_widget.is_none() && self.mouse_pressed {
                self.active_widget = Some(id);
            }
            color = Rgba::parse("red");
        }

        self.fill_rect(&area, z, &color);

        return !self.mouse_pressed // Mouse is released
            && self.active_widget == Some(id) // But this button is hot and active
            && self.hot_widget == Some(id);
    }

    fn draw_char<C: Color+Copy, D: Color+Copy>(&mut self, c: char, offset: V2<f32>, z: f32, color: &C, border: Option<&D>) {
        static BORDER: [V2<f32>; 8] =
            [V2(-1.0, -1.0), V2( 0.0, -1.0), V2( 1.0, -1.0),
             V2(-1.0,  0.0),                 V2( 1.0,  0.0),
             V2(-1.0,  1.0), V2( 0.0,  1.0), V2( 1.0,  1.0)];
        if let Some(img) = self.font_image(c) {
            if let Some(b) = border {
                // Put the border a tiny bit further in the z-buffer so it
                // won't clobber the text on the same layer.
                let border_z = z + 0.00001;
                for &d in BORDER.iter() {
                    self.draw_image(img, offset + d, border_z, b, &color::BLACK);
                }
            }
            self.draw_image(img, offset, z, color, &color::BLACK);
        }
    }

    fn char_width(&self, c: char) -> f32 {
        // Special case for space, the atlas image won't have a width.
        if c == ' ' { return (FONT_W / 2) as f32; }

        // Infer letter width from the cropped atlas image. (Use mx instead of
        // dim on the pos rectangle so that the left-side space will be
        // preserved and the letters are kept slightly apart.)
        if let Some(img) = self.font_image(c) {
            let width = self.image_data(img).pos.mx().0;
            return width;
        }

        // Not a valid letter.
        (FONT_W / 2) as f32
    }
}
