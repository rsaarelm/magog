use world::{Chart};
use calx;
use calx::{V2};
use calx::color;
use world::terrain;
use viewutil::{SCREEN_W, SCREEN_H, chart_to_view, cells_on_screen};
use tilecache;
use tilecache::tile;

pub fn draw_world<C: Chart>(chart: &C, ctx: &mut calx::Context) {
    for &pt in cells_on_screen().iter() {
        let screen_pos = chart_to_view(pt) + V2(SCREEN_W / 2, SCREEN_H / 2);
        //print!("{} ", screen_pos.0);

        let loc = *chart + pt;

        if loc.terrain() == terrain::Tree {
            ctx.draw_image(screen_pos, 0.4, tilecache::get(tile::TREE_TRUNK), &color::GREEN);
        } else {
            ctx.draw_image(screen_pos, 0.4, tilecache::get(tile::FLOOR), &color::ORANGE);
        }
    }
}
