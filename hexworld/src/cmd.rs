use std::cmp::{Ordering};
use std::hash::{Hash, Hasher};
use calx::{V2, HexGeom, astar_path_with, LatticeNode, Dir6};
use calx::{Projection, color};
use calx_ecs::{Entity};
use world::{World, Action, matches_mask, ComponentNum, Anim};
use spr::{Spr};
use ::{Sprite};

pub struct PathPos<'a> {
    pub world: &'a World,
    pub pos: V2<i32>,
}

impl<'a> PartialEq for PathPos<'a> { fn eq(&self, other: &PathPos<'a>) -> bool { self.pos == other.pos } }
impl<'a> Eq for PathPos<'a> {}
impl<'a> PartialOrd for PathPos<'a> { fn partial_cmp(&self, other: &PathPos<'a>) -> Option<Ordering> { self.pos.partial_cmp(&other.pos) } }
impl<'a> Ord for PathPos<'a> { fn cmp(&self, other: &PathPos<'a>) -> Ordering { self.pos.cmp(&other.pos) } }
impl<'a> Clone for PathPos<'a> { fn clone(&self) -> PathPos<'a> { PathPos { world: self.world, pos: self.pos } } }
impl<'a> Hash for PathPos<'a> { fn hash<H>(&self, state: &mut H) where H: Hasher { self.pos.hash(state) } }

impl<'a> PathPos<'a> {
    pub fn new(world: &'a World, pos: V2<i32>) -> PathPos<'a> {
        PathPos {
            world: world,
            pos: pos,
        }
    }
}

impl<'a> LatticeNode for PathPos<'a> {
    fn neighbors(&self) -> Vec<PathPos<'a>> {
        let mut ret = Vec::new();
        for i in Dir6::iter() {
            let pos = self.pos + i.to_v2();
            if self.world.terrain_at(pos).can_walk() {
                ret.push(PathPos::new(self.world, pos));
            }
        }
        ret
    }
}

pub fn find_path(ctx: &World, orig: V2<i32>, dest: V2<i32>) -> Option<Vec<V2<i32>>> {
    if let Some(path) = astar_path_with(
        |x, y| (x.pos-y.pos).hex_dist(), PathPos::new(ctx, orig), PathPos::new(ctx, dest), 1000) {
        // Skip the first step since that's where we are already.
        Some(path.iter().skip(1).map(|x| x.pos).collect())
    } else {
        None
    }
}

/// Return whether a path was found.
pub fn move_to(ctx: &mut World, e: Entity, dest: V2<i32>) -> bool {
    if let Some(path) = find_path(ctx, ctx.ecs.pos[e], dest) {
        let mob = &mut ctx.ecs.mob[e];
        // TODO: Use append when it's stable.
        for p in path.into_iter().map(|x| Action::MoveTo(x)) { mob.tasks.push(p); }
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

pub fn mob_at(ctx: &World, pos: V2<i32>) -> Option<Entity> {
    mobs(ctx).into_iter().filter(|&mob| ctx.ecs.pos[mob] == pos).next()
}
