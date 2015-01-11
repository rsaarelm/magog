use util::{V2};
use backend::{Context};

pub trait Drawable {
    fn draw(&self, ctx: &mut Context, offset: V2<i32>);
}

pub struct Translated<T> {
    inner: T,
    offset: V2<i32>,
}

impl<T: Drawable> Translated<T> {
    pub fn new(offset: V2<i32>, inner: T) -> Translated<T> {
        Translated {
            inner: inner,
            offset: offset
        }
    }
}

impl<T: Drawable> Drawable for Translated<T> {
    fn draw(&self, ctx: &mut Context, offset: V2<i32>) {
        self.inner.draw(ctx, self.offset + offset);
    }
}
