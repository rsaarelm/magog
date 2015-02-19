use std::num::Float;
use canvas::{Canvas, Image};
use util::{V2, Rect, Color, Rgba, color};
use ::{WidgetId};

/// Helper methods for canvas context that do not depend on the underlying
/// implementation details.
pub trait CanvasUtil {
    /// Draw a thick solid line on the canvas.
    fn draw_line<C: Color+Copy>(&mut self, width: u32, p1: V2<f32>, p2: V2<f32>, layer: f32, color: &C);
    /// Get the size of an atlas image.
    fn image_dim(&self, img: Image) -> V2<u32>;

    /// Draw a stored image on the canvas.
    fn draw_image<C: Color+Copy>(&mut self, img: Image,
        offset: V2<f32>, z: f32, color: &C, back_color: &C);

    /// Draw a filled rectangle
    fn fill_rect<C: Color+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: &C);

    // TODO: More specs
    fn button(&mut self, id: WidgetId, pos: V2<f32>, z: f32) -> bool;
}

impl CanvasUtil for Canvas {
    fn draw_line<C: Color+Copy>(&mut self, width: u32, p1: V2<f32>, p2: V2<f32>, layer: f32, color: &C) {
        let tex = self.solid_tex_coord();
        let v1 = p2 - p1;
        let v2 = V2(-v1.1, v1.0);

        let scalar = width as f32 / 2.0 * 1.0 / v2.dot(v2).sqrt();
        let v2 = v2 * scalar;

        let orig = p1.map(|x| x as f32);

        let ind0 = self.num_vertices();
        self.push_vertex(orig + v2, layer, tex, color, &color::BLACK);
        self.push_vertex(orig - v2, layer, tex, color, &color::BLACK);
        self.push_vertex(orig - v2 + v1, layer, tex, color, &color::BLACK);
        self.push_vertex(orig + v2 + v1, layer, tex, color, &color::BLACK);
        self.push_triangle(ind0, ind0 + 1, ind0 + 2);
        self.push_triangle(ind0, ind0 + 2, ind0 + 3);
    }

    fn image_dim(&self, img: Image) -> V2<u32> {
        self.image_data(img).pos.1.map(|x| x as u32)
    }

    fn draw_image<C: Color+Copy>(&mut self, img: Image,
        offset: V2<f32>, z: f32, color: &C, back_color: &C) {
        let mut pos;
        let mut tex;
        {
            let data = self.image_data(img);
            pos = data.pos + offset;
            tex = data.tex;
        }

        let ind0 = self.num_vertices();

        self.push_vertex(pos.top_left(), z, tex.top_left(), color, back_color);
        self.push_vertex(pos.top_right(), z, tex.top_right(), color, back_color);
        self.push_vertex(pos.bottom_right(), z, tex.bottom_right(), color, back_color);
        self.push_vertex(pos.bottom_left(), z, tex.bottom_left(), color, back_color);

        self.push_triangle(ind0, ind0 + 1, ind0 + 2);
        self.push_triangle(ind0, ind0 + 2, ind0 + 3);
    }

    fn fill_rect<C: Color+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: &C) {
        let tex = self.solid_tex_coord();
        let ind0 = self.num_vertices();

        self.push_vertex(rect.top_left(), z, tex, color, &color::BLACK);
        self.push_vertex(rect.top_right(), z, tex, color, &color::BLACK);
        self.push_vertex(rect.bottom_right(), z, tex, color, &color::BLACK);
        self.push_vertex(rect.bottom_left(), z, tex, color, &color::BLACK);

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
}
