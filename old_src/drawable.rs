use calx::{Context, V2};

pub trait Drawable {
    fn draw(&self, ctx: &mut Context, offset: V2<int>);
}

pub struct Translated<T> {
    inner: T,
    offset: V2<int>,
}

impl<T: Drawable> Translated<T> {
    pub fn new(offset: V2<int>, inner: T) -> Translated<T> {
        Translated {
            inner: inner,
            offset: offset
        }
    }
}

impl<T: Drawable> Drawable for Translated<T> {
    fn draw(&self, ctx: &mut Context, offset: V2<int>) {
        self.inner.draw(ctx, self.offset + offset);
    }
}
