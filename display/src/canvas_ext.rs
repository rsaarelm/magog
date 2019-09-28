use crate::cache;
use crate::view::ScreenVector;
use euclid::default::Point2D;
use euclid::vec2;
use vitral::Canvas;
use world::Icon;

/// Magog-specific extensions to Canvas API.
pub trait CanvasExt {
    fn draw_entity(&mut self, pos: Point2D<i32>, icon: Icon);

    fn draw_item_icon(&mut self, pos: Point2D<i32>, icon: Icon) {
        self.draw_entity(pos + vec2(0, -2), icon); // TODO
    }
}

impl CanvasExt for Canvas<'_> {
    fn draw_entity(&mut self, pos: Point2D<i32>, icon: Icon) {
        let pos = ScreenVector::from_untyped(pos.to_vector());
        for splat in &cache::entity(icon)[0] {
            self.draw_image_2color(
                &splat.image,
                (pos - splat.offset).to_point().to_untyped(),
                splat.color,
                splat.back_color,
            );
        }
    }
}
