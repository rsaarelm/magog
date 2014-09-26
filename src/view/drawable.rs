use cgmath::{Vector, Vector2};
use calx::{Context};

pub trait Drawable {
    fn draw(&self, ctx: &mut Context, offset: (int, int));
}

pub struct Translated<T> {
    inner: T,
    offset: (int, int),
}

impl<T: Drawable> Translated<T> {
    pub fn new(offset: (int, int), inner: T) -> Translated<T> {
        Translated {
            inner: inner,
            offset: offset
        }
    }
}

impl<T: Drawable> Drawable for Translated<T> {
    fn draw(&self, ctx: &mut Context, (offset_x, offset_y): (int, int)) {
        let (x, y) = self.offset;
        self.inner.draw(ctx, (x + offset_x, y + offset_y))
    }
}
