use std::num::Float;
use canvas::Canvas;
use util::{V2, Color};

/// Helper methods for canvas context that do not depend on the underlying
/// implementation details.
pub trait CanvasUtil {
    fn draw_line<C: Color+Clone>(&mut self, width: u32, p1: V2<i32>, p2: V2<i32>, layer: f32, color: &C);
}

impl CanvasUtil for Canvas {
    fn draw_line<C: Color+Clone>(&mut self, width: u32, p1: V2<i32>, p2: V2<i32>, layer: f32, color: &C) {
        let v1 = (p2 - p1).map(|x| x as f32);
        let v2 = V2(-v1.1, v1.0);

        let scalar = width as f32 / 2.0 * 1.0 / v2.dot(v2).sqrt();
        let v2 = v2 * scalar;

        let orig = p1.map(|x| x as f32);

        self.draw_tri(
            layer,
            [(orig + v2).map(|x| x as i32),
             (orig - v2).map(|x| x as i32),
             (orig + v2 + v1).map(|x| x as i32)],
            [color.clone(), color.clone(), color.clone()]);
        self.draw_tri(
            layer,
            [(orig - v2).map(|x| x as i32),
             (orig - v2 + v1).map(|x| x as i32),
             (orig + v2 + v1).map(|x| x as i32)],
            [color.clone(), color.clone(), color.clone()]);
    }
}
