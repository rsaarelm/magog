use std::slice::Iter;
use calx::{color, ToColor, V2, Dir6, clamp};
use calx::backend::{Canvas, CanvasUtil};
use world::{Location, Unchart};
use viewutil::{FX_Z, chart_to_screen};
use tilecache;
use tilecache::tile;

trait WorldSprite {
    fn update(&mut self);
    fn is_alive(&self) -> bool;

    fn footprint<'a>(&'a self) -> Iter<'a, Location>;
    // XXX: Locked to the type of iterator Vecs return for now. It's assumed
    // that implementers use a Vec to cache the footprint points internally.

    fn draw(&self, chart: &Location, ctx: &mut Canvas);
    // XXX: Can't parametrize to Unchart since trait objects can't have
    // parameterized methods.
}

pub struct WorldSprites {
    sprites: Vec<Box<WorldSprite + 'static>>,
}

impl WorldSprites {
    pub fn new() -> WorldSprites {
        WorldSprites {
            sprites: vec!(),
        }
    }

    pub fn add(&mut self, spr: Box<WorldSprite + 'static>) {
        self.sprites.push(spr);
    }

    pub fn draw<F>(&self, is_visible: F, chart: &Location, ctx: &mut Canvas)
        where F: Fn(V2<i32>) -> bool {
        // XXX: Ineffective if there are many sprites outside the visible
        // area.
        for s in self.sprites.iter() {
            for &loc in s.footprint() {
                if chart.chart_pos(loc).map_or(false, |p| is_visible(p)) {
                    s.draw(chart, ctx);
                    break;
                }
            }
        }
    }

    pub fn update(&mut self) {
        for spr in self.sprites.iter_mut() { spr.update(); }
        self.sprites.retain(|spr| spr.is_alive());
    }
}

pub struct BeamSprite {
    p1: Location,
    p2: Location,
    life: i32,
    footprint: Vec<Location>,
}

impl BeamSprite {
    pub fn new(p1: Location, p2: Location, life: i32) -> BeamSprite {
        BeamSprite {
            p1: p1,
            p2: p2,
            life: life,
            // TODO: Generate intervening points into the footprint. With this
            // footprint you can't see the beam unless either the start or the
            // end point is visible.
            footprint: vec![p1, p2],
        }
    }
}

impl WorldSprite for BeamSprite {
    fn update(&mut self) { self.life -= 1; }
    fn is_alive(&self) -> bool { self.life >= 0 }
    fn footprint<'a>(&'a self) -> Iter<'a, Location> {
        self.footprint.iter()
    }
    fn draw(&self, chart: &Location, ctx: &mut Canvas) {
        if let (Some(p1), Some(p2)) = (chart.chart_pos(self.p1), chart.chart_pos(self.p2)) {
            let pixel_adjust = V2(0.0, -2.0);
            ctx.draw_line(2,
                chart_to_screen(p1) + pixel_adjust,
                chart_to_screen(p2) + pixel_adjust,
                FX_Z, &color::ORANGE);
        }
    }
}

pub struct GibSprite {
    loc: Location,
    life: i32,
    footprint: Vec<Location>,
}

impl GibSprite {
    pub fn new(loc: Location) -> GibSprite {
        GibSprite {
            loc: loc,
            life: 11,
            footprint: vec![loc],
        }
    }
}

impl WorldSprite for GibSprite {
    fn update(&mut self) { self.life -= 1; }
    fn is_alive(&self) -> bool { self.life >= 0 }
    fn footprint<'a>(&'a self) -> Iter<'a, Location> { self.footprint.iter() }
    fn draw(&self, chart: &Location, ctx: &mut Canvas) {
        if let Some(p) = chart.chart_pos(self.loc) {
            // TODO: Robust anim cycle with clamping.
            let idx = tile::SPLATTER + ((11 - self.life) / 3) as usize;
            ctx.draw_image(tilecache::get(idx), chart_to_screen(p), FX_Z, &color::RED, &color::BLACK);
        }
    }
}

pub struct ExplosionSprite {
    loc: Location,
    life: i32,
    footprint: Vec<Location>,
}

impl ExplosionSprite {
    pub fn new(loc: Location) -> ExplosionSprite {
        ExplosionSprite {
            loc: loc,
            life: 12,
            footprint: Dir6::iter().map(|d| loc + d.to_v2()).chain(Some(loc).into_iter()).collect(),
        }
    }
}

impl WorldSprite for ExplosionSprite {
    fn update(&mut self) { self.life -= 1; }
    fn is_alive(&self) -> bool { self.life >= 0 }
    fn footprint<'a>(&'a self) -> Iter<'a, Location> { self.footprint.iter() }
    fn draw(&self, chart: &Location, ctx: &mut Canvas) {
        if let Some(p) = chart.chart_pos(self.loc) {
            let center = chart_to_screen(p);

            // Growing in [0.0, 1.0].
            let t = clamp(0.0, 1.0, (12 - self.life) as f32 / 4.0);
            let t2 = clamp(0.0, 1.0, (8 - self.life) as f32 / 4.0);

            hexagon(ctx, center, FX_Z, &color::ORANGE, &color::RED, t2 * 1.8, t * 2.0);
        }
    }
}

fn hexagon<C: ToColor+Copy>(ctx: &mut Canvas, center: V2<f32>, z: f32,
                            color_inner: &C, color_outer: &C,
                            inner_radius: f32, outer_radius: f32) {
    let tex = ctx.solid_tex_coord();
    static VERTICES: [V2<f32>; 6] = [
        V2(-8.0,  -4.0),
        V2( 0.0,  -8.0),
        V2( 8.0,  -4.0),
        V2( 8.0,   4.0),
        V2( 0.0,   8.0),
        V2(-8.0,   4.0),
    ];

    // Inner vertices
    let inner = ctx.num_vertices();
    for p in VERTICES.iter() {
        ctx.push_vertex(center + (*p * inner_radius), z, tex, color_inner, &color::BLACK)
    }

    // Outer vertices
    let outer = ctx.num_vertices();
    for p in VERTICES.iter() {
        ctx.push_vertex(center + (*p * outer_radius), z, tex, color_outer, &color::BLACK)
    }

    // Polygon pushing
    for i in 0..6 {
        let i2 = (i + 1) % 6;
        ctx.push_triangle(inner + i, outer + i2, inner + i2);
        ctx.push_triangle(inner + i, outer + i, outer + i2);
    }
}
