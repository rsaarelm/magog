use cgmath::vector::{Vector, Vector2};
use calx::engine::{Engine};

pub trait Drawable {
    fn draw(&self, ctx: &mut Engine, offset: &Vector2<f32>);
}

pub struct Translated<T> {
    inner: T,
    offset: Vector2<f32>,
}

impl<T: Drawable> Translated<T> {
    pub fn new(offset: Vector2<f32>, inner: T) -> Translated<T> {
        Translated {
            inner: inner,
            offset: offset
        }
    }
}

impl<T: Drawable> Drawable for Translated<T> {
    fn draw(&self, ctx: &mut Engine, offset: &Vector2<f32>) {
        self.inner.draw(ctx, &offset.add_v(&self.offset));
    }
}
