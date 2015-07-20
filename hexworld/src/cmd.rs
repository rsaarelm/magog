use calx::{V2, HexGeom, Dir6};
use calx_ecs::{Entity};
use world::{World, Goal};
use rule;
use path;

pub fn move_to(ctx: &mut World, e: Entity, dest: V2<i32>) -> bool {
    ctx.ecs.mob[e].goals.clear();

    ctx.ecs.mob[e].goals.push(Goal::MoveTo(dest));

    // Return whether a path can be found.
    path::towards(ctx, e, dest).is_some()
}

pub fn attack(ctx: &mut World, e: Entity, enemy: Entity) -> bool {
    ctx.ecs.mob[e].goals.clear();

    ctx.ecs.mob[e].goals.push(Goal::Attack(enemy));

    // Return whether a path can be found.
    let dest = ctx.ecs.pos[e];
    path::towards(ctx, e, dest).is_some()
}

pub fn update_mob(ctx: &mut World, e: Entity) {
    let old_pos = ctx.ecs.pos[e];

    if ctx.ecs.mob[e].action_delay > 0 {
        ctx.ecs.mob[e].action_delay -= 1;
    } else {
        match ctx.ecs.mob[e].goals.first() {
            None => {}
            Some(&Goal::MoveTo(pos)) => {
                if old_pos == pos {
                    // Already there, remove goal.
                    ctx.ecs.mob[e].goals.remove(0);
                } else {
                    match path::towards(ctx, e, pos) {
                        Some(dir) => { rule::step(ctx, e, dir); }
                        None => { ctx.ecs.mob[e].goals.remove(0); }
                    }
                }
            }
            Some(&Goal::Attack(mob)) => {
                if !ctx.ecs.contains(mob) {
                    // Target is gone.
                    ctx.ecs.mob[e].goals.remove(0);
                } else {
                    let pos = ctx.ecs.pos[mob];
                    let vec_to_enemy = pos - old_pos;
                    if vec_to_enemy.hex_dist() <= 1 {
                        // Within punching distance.
                        rule::melee(ctx, e, Dir6::from_v2(vec_to_enemy));
                    } else {
                        // Too far, move closer.
                        match path::towards(ctx, e, pos) {
                            Some(dir) => { rule::step(ctx, e, dir); }
                            None => { ctx.ecs.mob[e].goals.remove(0); }
                        }
                    }
                }
            }
            _ => {
                unimplemented!();
            }
        }
    }
}
