use calx::{V2, HexGeom, astar_path_with};
use calx::{Projection, color};
use calx_ecs::{Entity};
use world::{World, Action, matches_mask, ComponentNum, Anim};
use spr::{Spr};
use ::{PathPos, Sprite};

/// Return whether a path was found.
pub fn move_to(ctx: &mut World, e: Entity, dest: V2<i32>) -> bool {
    let pos = ctx.ecs.pos[e];
    if let Some(path) = astar_path_with(
        |x, y| (x.0-y.0).hex_dist(), PathPos(pos), PathPos(dest), 1000) {
        let mob = &mut ctx.ecs.mob[e];
        // TODO: Use append when it's stable.
        for p in path.into_iter().map(|x| Action::MoveTo(x.0)) { mob.tasks.push(p); }
        true
    } else {
        false
    }
}

pub fn player(ctx: &World) -> Option<Entity> {
    mobs(ctx).into_iter().filter(|&mob| is_player(ctx, mob)).next()
}

pub fn is_player(ctx: &World, mob: Entity) -> bool {
    ctx.ecs.desc[mob].icon == Spr::Avatar
}

pub fn mobs(ctx: &World) -> Vec<Entity> {
    ctx.ecs.iter().filter(|&&e| matches_mask(&ctx.ecs, e, build_mask!(desc, mob, pos))).cloned().collect()
}

pub fn sprite(ctx: &World, e: Entity, p: &Projection) -> Option<Sprite> {
    if !ctx.ecs.pos.contains(e) || !ctx.ecs.desc.contains(e) { return None; }

    let cell_pos = ctx.ecs.pos[e];
    let spr = ctx.ecs.desc[e].icon;
    let color = ctx.ecs.desc[e].color;

    let default_anim = Anim::Standstill;
    let anim = if ctx.ecs.mob.contains(e) { &ctx.ecs.mob[e].anim } else { &default_anim };

    let draw_pos = anim.get_pos(cell_pos, ctx.anim_t, p);

    Some(Sprite::new(spr, draw_pos, 0, color, color::BLACK))
}
