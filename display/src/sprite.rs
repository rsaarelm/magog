use backend::MagogContext;
use brush::Brush;
use calx_alg::lerp;
use calx_color::{color, Rgba};
use euclid::{Point2D, point2};
use render::Layer;
use std::cmp::Ordering;
use std::rc::Rc;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Coloring {
    /// Use map memory coloring for this sprite.
    MapMemory,
    /// Use the darkness level in [0.0, 1.0] for this sprite.
    Shaded(f32),
}

impl Default for Coloring {
    fn default() -> Self { Coloring::Shaded(1.0) }
}

impl Eq for Coloring {}

impl Coloring {
    // XXX: Maybe we should still be using calx::Rgba here?
    pub fn apply(self, fore: Rgba, back: Rgba) -> (Rgba, Rgba) {
        match self {
            Coloring::MapMemory => (Rgba::from(0x0804_00ffu32), Rgba::from(0x3322_00ff)),
            Coloring::Shaded(a) => (lerp(color::BLACK, fore, a), lerp(color::BLACK, back, a)),
        }
    }
}

/// Drawable display element.
///
/// Sprites are basically a way to defer somewhat complex draw instructions. The reason they exist
/// is that scene draw order is not necessarily trivially reflectable in scene data traversal, so
/// emitting sprites and then sorting them is the simplest way to go ahead.
#[derive(Clone, PartialEq)]
pub struct Sprite {
    pub layer: Layer,
    // XXX: Not using Point2D<f32> because floats don't have Ord.
    pub offset: [i32; 2],

    // TODO: Replace this with a generic "Drawable" trait object once we start having other things
    // than frames as sprites.
    pub brush: Rc<Brush>,
    pub frame_idx: usize,
    pub color: Coloring,
}

impl Sprite {
    pub fn new(layer: Layer, offset: Point2D<f32>, brush: Rc<Brush>) -> Sprite {
        let offset = [offset.x as i32, offset.y as i32];
        Sprite {
            layer,
            offset,
            brush,
            frame_idx: 0,
            color: Default::default(),
        }
    }

    pub fn idx(mut self, frame_idx: usize) -> Sprite {
        self.frame_idx = frame_idx;
        self
    }

    pub fn color(mut self, color: Coloring) -> Sprite {
        self.color = color;
        self
    }
}

impl Eq for Sprite {}

impl Ord for Sprite {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.layer, self.offset[1]).cmp(&(other.layer, other.offset[1]))
    }
}

impl PartialOrd for Sprite {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Sprite {
    pub fn draw<C: MagogContext>(&self, ui: &mut C) {
        let pos = point2(self.offset[0] as f32, self.offset[1] as f32);
        for splat in &self.brush[self.frame_idx] {
            let (fore, back) = self.color.apply(splat.color, splat.back_color);
            ui.draw_image_2color(&splat.image, pos - splat.offset, fore.into(), back.into());
        }
    }
}
