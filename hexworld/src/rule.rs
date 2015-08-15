/*! Low-level game object rules. */

use calx::{V2, Dir6, HexGeom, color, Projection};
use calx_ecs::{Entity};
use world::{World, matches_mask, ComponentNum, Anim, Tween};
use brush::{Brush};
use ::{Sprite};

pub fn is_player(ctx: &World, mob: Entity) -> bool {
    ctx.ecs.desc[mob].icon == Brush::Avatar
}

pub fn mobs(ctx: &World) -> Vec<Entity> {
    ctx.ecs.iter().filter(|&&e| matches_mask(&ctx.ecs, e, build_mask!(desc, mob, pos))).cloned().collect()
}

pub fn sprite(ctx: &World, e: Entity, p: &Projection) -> Option<Sprite> {
    // XXX: Can't recolorize the sprite after it's returned from here since
    // the Drawable Box in Sprite is opaque.

    if !ctx.ecs.pos.contains(e) || !ctx.ecs.desc.contains(e) { return None; }

    let cell_pos = ctx.ecs.pos[e];
    let brush = ctx.ecs.desc[e].icon;
    let color = ctx.ecs.desc[e].color;

    let default_anim = Anim::Standstill;
    let anim = if ctx.ecs.mob.contains(e) { &ctx.ecs.mob[e].anim } else { &default_anim };

    let draw_pos = anim.get_pos(cell_pos, ctx.anim_t, p);

    Some(Sprite::new_spr(brush, 0, color, color::BLACK, draw_pos, 0))
}

pub fn mob_at(ctx: &World, pos: V2<i32>) -> Option<Entity> {
    // TODO: Needs optimization!
    mobs(ctx).into_iter().filter(|&mob| ctx.ecs.pos[mob] == pos).next()
}

pub fn can_enter(ctx: &World, e: Entity, pos: V2<i32>) -> Option<BlockCause> {
    if !ctx.terrain_at(pos).can_walk() { return Some(BlockCause::Terrain); }
    match mob_at(ctx, pos) {
        Some(m) if m != e => { return Some(BlockCause::Mob(m)); }
        _ => { return None; }
    }
}

pub enum BlockCause {
    Terrain,
    Mob(Entity),
}

pub fn step(ctx: &mut World, e: Entity, dir: Dir6) -> Option<BlockCause> {
    let old_pos = ctx.ecs.pos[e];

    let new_pos = old_pos + dir.to_v2();

    let cause = can_enter(ctx, e, new_pos);

    if cause.is_some() {
        return cause;
    } else {
        let mob = &mut ctx.ecs.mob[e];
        let move_delay = 6;
        mob.action_delay = move_delay;
        mob.anim = Anim::Move(
            Tween::new(ctx.anim_t, old_pos, move_delay));
    }

    ctx.ecs.pos[e] = new_pos;

    None
}

/// Make an entity perform a melee action that produces an attack.
pub fn melee(ctx: &mut World, e: Entity, dir: Dir6) {
    let old_pos = ctx.ecs.pos[e];
    let new_pos = old_pos + dir.to_v2();

    if let Some(target) = mob_at(ctx, new_pos) {
        {
            let mob = &mut ctx.ecs.mob[e];
            let move_delay = 6;
            mob.action_delay = move_delay;
            mob.anim = Anim::Attack(Tween::new(ctx.anim_t, new_pos, move_delay));
        }

        damage(ctx, target);
    }
}

/// Deal damage to a target.
pub fn damage(ctx: &mut World, target: Entity) {
    // TODO: Damage the target instead of just destroying it. Will also
    // involve adding stuff to the API like the amount of damage.

    // TODO: Also, don't delete entities from the ECS mid-update, have a dead
    // flag instead. There *will* be more complex logic that will want to keep
    // doing stuff to the entity.
    ctx.ecs.remove(target);
}

pub fn ready_to_act(ctx: &World, e: Entity) -> bool {
    ctx.ecs.mob[e].action_delay == 0
}

/// Return whether entity e considers entity g an enemy to be attacked.
pub fn is_enemy_of(ctx: &World, e: Entity, g: Entity) -> bool {
    ctx.ecs.mob.contains(e)
        && ctx.ecs.mob.contains(g)
        && is_player(ctx, e) != is_player(ctx, g)
}

/// List currently visible enemy entities to given entity.
pub fn visible_enemies(ctx: &World, e: Entity) -> Vec<Entity> {
    // TODO: Needs optimization!
    // TODO: Line of sight.
    let pos = ctx.ecs.pos[e];
    let sight_range = 6; // TODO: Make bigger? From mob stats?
    let mut ret = Vec::new();
    for m in mobs(ctx) {
        if is_enemy_of(ctx, e, m) && (ctx.ecs.pos[m] - pos).hex_dist() <= sight_range {
            ret.push(m);
        }
    }

    ret
}
