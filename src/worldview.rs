use std::collections::HashMap;
use calx::{V2, Rgba, timing, Kernel};
use calx::color::*;
use calx::backend::{Canvas, CanvasUtil, Image};
use content::{Brush};
use world::{Location, Chart};
use world::{FovStatus};
use world::{Entity};
use world::{Light};
use viewutil::{chart_to_screen, cells_on_screen};
use viewutil::{FLOOR_Z, BLOCK_Z};
use drawable::{Drawable};
use gamescreen::{Blink};
use render_terrain::{self, Angle, Purpose};

pub fn draw_world<C: Chart+Copy>(chart: &C, ctx: &mut Canvas, damage_timers: &HashMap<Entity, (Blink, u32)>) {
    for pt in cells_on_screen() {
        let screen_pos = chart_to_screen(pt);
        let loc = *chart + pt;
        let cell_drawable = CellDrawable::new(
            loc, 0, loc.fov_status(), loc.light(), damage_timers);
        cell_drawable.draw(ctx, screen_pos);
    }
}

/// Drawable representation of a single map location.
pub struct CellDrawable<'a> {
    pub loc: Location,
    pub depth: i32,
    pub fov: Option<FovStatus>,
    pub light: Light,
    damage_timers: &'a HashMap<Entity, (Blink, u32)>,
}

impl<'a> Drawable for CellDrawable<'a> {
    fn draw(&self, ctx: &mut Canvas, offset: V2<f32>) {
        match self.fov {
            Some(_) => {
                self.draw_cell(ctx, offset)
            }
            None => {
                self.draw_image(ctx, Brush::BlankFloor.get(0), offset, FLOOR_Z, BLACK, BLACK);
            }
        }
    }
}

impl<'a> CellDrawable<'a> {
    pub fn new(
        loc: Location,
        depth: i32,
        fov: Option<FovStatus>,
        light: Light,
        damage_timers: &'a HashMap<Entity, (Blink, u32)>) -> CellDrawable<'a> {
        CellDrawable {
            loc: loc,
            depth: depth,
            fov: fov,
            light: light,
            damage_timers: damage_timers,
        }
    }

    fn draw_image(&'a self, ctx: &mut Canvas, img: Image, offset: V2<f32>, z: f32,
                  color: Rgba, back_color: Rgba) {
        let (mut color, mut back_color) = match self.fov {
            // XXX: Special case for the solid-black objects that are used to
            // block out stuff to not get recolored. Don't use total black as
            // an actual object color, have something like #010101 instead.
            Some(FovStatus::Remembered) if color != BLACK => (BLACK, Rgba::from(0x332200FF)),
            _ => (color, back_color),
        };
        if self.fov == Some(FovStatus::Seen) {
            color = self.light.apply(color);
            back_color = self.light.apply(back_color);
        }
        if self.depth != 0 && self.fov != Some(FovStatus::Seen) { return; }

        ctx.draw_image(img, offset, z, color, back_color);
    }

    fn draw_cell(&'a self, ctx: &mut Canvas, offset: V2<f32>) {
        let visible = self.fov == Some(FovStatus::Seen);
        let k = Kernel::new(|loc| loc.terrain(), self.loc);
        render_terrain::render(&k, |img, angle, purpose, fore, back| {
                 let z = match angle {
                     Angle::Up => FLOOR_Z,
                     _ => BLOCK_Z
                 };
                 if visible || purpose == Purpose::Element {
                     self.draw_image(ctx, img, offset, z, fore, back)
                 }
        });

        if visible {
            // Sort mobs on top of items for drawing.
            let mut es = self.loc.entities();
            es.sort_by(|a, b| a.is_mob().cmp(&b.is_mob()));
            for e in es.iter() {
                self.draw_entity(ctx, offset, e);
            }
        }
    }

    fn draw_entity(&'a self, ctx: &mut Canvas, offset: V2<f32>, entity: &Entity) {
        let body_pos =
            if entity.is_bobbing() {
                offset + *(timing::cycle_anim(
                        0.3f64,
                        &[V2(0.0, 0.0), V2(0.0, -1.0)]))
            } else { offset };

        if let Some((brush, mut color)) = entity.get_brush() {
            let mut back_color = BLACK;

            // Damage blink animation.
            if let Some(&(ref b, ref t)) = self.damage_timers.get(entity) {
                match b {
                    &Blink::Damaged => {
                        if t % 2 == 0 {
                            color = WHITE;
                            back_color = WHITE;
                        } else {
                            color = BLACK;
                            back_color = BLACK;
                        }
                    }

                    &Blink::Threat => {
                        color = RED;
                        back_color = WHITE;
                    }
                }
            }

            if entity.is_item() {
                // Blink pickups intermittently to draw attention.
                if timing::spike(1.5, 0.1) {
                    color = WHITE;
                    back_color = WHITE;
                }
            }

            // The serpent mob has an extra mound element that
            // doesn't bob along with the main body.
            if brush == Brush::Serpent {
                // Body
                self.draw_image(ctx, brush.get(0), body_pos, BLOCK_Z, color, back_color);
                // Ground mound, doesn't bob.
                self.draw_image(ctx, brush.get(1), offset, BLOCK_Z, color, back_color);
                return;
            } else {
                self.draw_image(ctx, brush.get(0), body_pos, BLOCK_Z, color, back_color);
            }
        }
    }
}
