use calx::{V2};
use calx::backend::{Canvas};

pub trait Drawable {
    fn draw(&self, ctx: &mut Canvas, offset: V2<f32>);
}

pub struct Translated<T> {
    inner: T,
    offset: V2<f32>,
}

impl<T: Drawable> Translated<T> {
    pub fn new(offset: V2<f32>, inner: T) -> Translated<T> {
        Translated {
            inner: inner,
            offset: offset
        }
    }
}

impl<T: Drawable> Drawable for Translated<T> {
    fn draw(&self, ctx: &mut Canvas, offset: V2<f32>) {
        self.inner.draw(ctx, self.offset + offset);
    }
}
