use std::num::Float;
use canvas::{Canvas, Image};
use util::{V2, Color, color};

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

        self.push_vertex(pos.p0(), z, tex.p0(), color, back_color);
        self.push_vertex(pos.p1(), z, tex.p1(), color, back_color);
        self.push_vertex(pos.p2(), z, tex.p2(), color, back_color);
        self.push_vertex(pos.p3(), z, tex.p3(), color, back_color);

        self.push_triangle(ind0, ind0 + 1, ind0 + 2);
        self.push_triangle(ind0, ind0 + 2, ind0 + 3);
    }
}
