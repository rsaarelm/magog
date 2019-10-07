use crate::cache;
use crate::view::ScreenVector;
use euclid::default::Point2D;
use euclid::vec2;
use vitral::Canvas;
use vitral::{color, Align};
use world::Icon;

/// Magog-specific extensions to Canvas API.
pub trait CanvasExt {
    fn draw_entity(&mut self, pos: Point2D<i32>, icon: Icon, count: u32);

    fn draw_item_icon(&mut self, pos: Point2D<i32>, icon: Icon, count: u32) {
        self.draw_entity(pos + vec2(0, -2), icon, count); // TODO
    }
}

impl CanvasExt for Canvas<'_> {
    fn draw_entity(&mut self, pos: Point2D<i32>, icon: Icon, count: u32) {
        let vec = ScreenVector::from_untyped(pos.to_vector());
        for splat in &cache::entity(icon)[0] {
            self.draw_image_2color(
                &splat.image,
                (vec - splat.offset).to_point().to_untyped(),
                splat.color,
                splat.back_color,
            );
        }

        if count > 1 {
            self.draw_outline_text(
                &*cache::tiny_font(),
                pos - vec2(9, 9),
                Align::Left,
                color::SILVER,
                color::GRAY2,
                &format!("{}", count),
            );
        }
    }
}
