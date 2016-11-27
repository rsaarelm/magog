use std::cmp::Ordering;
use euclid::Point2D;
use calx_resource::Resource;
use world::Brush;
use backend;
use render::Layer;

/// Drawable display element.
///
/// Sprites are basically a way to defer somewhat complex draw instructions. The reason they exist
/// is that scene draw order is not necessarily trivially reflectable in scene data traversal, so
/// emitting sprites and then sorting them is the simplest way to go ahead.
#[derive(Clone, PartialEq, Eq)]
pub struct Sprite {
    pub layer: Layer,
    // XXX: Not using Point2D<f32> because floats don't have Ord.
    pub offset: [i32; 2],

    // TODO: Replace this with a generic "Drawable" trait object once we start having other things
    // than frames as sprites.
    pub brush: Resource<Brush>,
    pub frame_idx: usize,
}

impl Ord for Sprite {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.layer, self.offset[1]).cmp(&(other.layer, other.offset[1]))
    }
}

impl PartialOrd for Sprite {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Sprite {
    pub fn draw(&self, ui: &mut backend::UI) {
        let pos = Point2D::new(self.offset[0] as f32, self.offset[1] as f32);
        for splat in &self.brush[self.frame_idx] {
            ui.draw_image(&splat.image, pos - splat.offset, splat.color);
        }
    }
}
