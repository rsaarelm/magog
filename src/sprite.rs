use std::cmp::Ordering;
use euclid::Point2D;
use calx_resource::Resource;
use world::Brush;
use backend;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// Draw layer for visual map elements.
pub enum Layer {
    /// Floor sprites, below other map forms.
    Floor,
    /// Blood splatters etc. on floor.
    Decal,
    /// Small items on floor,.
    Items,
    /// Large map objects, walls, most entities etc.
    Object,
    /// Transient effects shown on top of other map content.
    Effect,
    /// Text captions for map elements, on top of everything else. 
    Text,
}

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
        (self.layer, self.offset[1], self.offset[0])
            .cmp(&(other.layer, other.offset[1], other.offset[0]))
    }
}

impl PartialOrd for Sprite {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Sprite {
    pub fn draw(&self, context: &mut backend::Context) {
        let pos = Point2D::new(self.offset[0] as f32, self.offset[1] as f32);
        for splat in self.brush[self.frame_idx].iter() {
            context.draw_image(splat.image, pos - splat.offset, splat.color);
        }
    }
}
