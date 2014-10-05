use world::{Chart};
use calx;
use calx::{V2};
use calx::color;
use viewutil;
use tilecache;

pub fn draw_world<C: Chart>(chart: &C, ctx: &mut calx::Context) {
    ctx.draw_image(V2(32, 32), 0.4, tilecache::get(tilecache::tile::AVATAR), &color::ORANGE);
    // TODO
}
