use std::cmp::{Ordering};
use std::hash::{Hash, Hasher};
use calx::{V2, astar_path_with, LatticeNode, Dir6, HexGeom};
use calx_ecs::{Entity};
use world::{World};

struct PathPos<'a> {
    world: &'a World,
    pos: V2<i32>,
}

impl<'a> PartialEq for PathPos<'a> { fn eq(&self, other: &PathPos<'a>) -> bool { self.pos == other.pos } }
impl<'a> Eq for PathPos<'a> {}
impl<'a> PartialOrd for PathPos<'a> { fn partial_cmp(&self, other: &PathPos<'a>) -> Option<Ordering> { self.pos.partial_cmp(&other.pos) } }
impl<'a> Ord for PathPos<'a> { fn cmp(&self, other: &PathPos<'a>) -> Ordering { self.pos.cmp(&other.pos) } }
impl<'a> Clone for PathPos<'a> { fn clone(&self) -> PathPos<'a> { PathPos { world: self.world, pos: self.pos } } }
impl<'a> Hash for PathPos<'a> { fn hash<H>(&self, state: &mut H) where H: Hasher { self.pos.hash(state) } }

impl<'a> PathPos<'a> {
    fn new(world: &'a World, pos: V2<i32>) -> PathPos<'a> {
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

/// Return the step sequence to lead to target point if one can be found.
pub fn find(ctx: &World, orig: V2<i32>, dest: V2<i32>) -> Option<Vec<V2<i32>>> {
    if let Some(path) = astar_path_with(
        |x, y| (x.pos-y.pos).hex_dist(), PathPos::new(ctx, orig), PathPos::new(ctx, dest), 1000) {
        // Skip the first step since that's where we are already.
        Some(path.iter().skip(1).map(|x| x.pos).collect())
    } else {
        None
    }
}

/// Best movement direction to get towards destination.
pub fn towards(ctx: &World, e: Entity, dest: V2<i32>) -> Option<Dir6> {
    // XXX: A*-pathing anew for every step, VERY wasteful.
    if let Some(node) =  find(ctx, ctx.ecs.pos[e], dest).map_or(None, |x| x.first().map(|&y| y)) {
        Some(Dir6::from_v2(node - ctx.ecs.pos[e]))
    } else {
        None
    }
}
