//! Logic for movement and game world space
use crate::{
    stats::{Intrinsic, Status},
    ActionOutcome, Location, Sector, World,
};
use calx::{Clamp, Dir6, RngExt};
use calx_ecs::Entity;
use rand::Rng;

impl World {
    /// Mark an entity as dead, but don't remove it from the system yet.
    pub(crate) fn kill_entity(&mut self, e: Entity) {
        if self.count(e) > 1 {
            self.ecs_mut().stacking[e].count -= 1;
        } else {
            self.spatial.remove(e);
        }
    }

    /// Remove an entity from the system.
    ///
    /// You generally do not want to call this directly. Mark the entity as dead and it will be
    /// removed at the end of the turn.
    pub(crate) fn remove_entity(&mut self, e: Entity) { self.ecs.remove(e); }

    pub(crate) fn place_entity(&mut self, e: Entity, mut loc: Location) {
        if self.is_item(e) {
            loc = self.empty_item_drop_location(loc);
        }
        self.set_entity_location(e, loc);
        self.after_entity_moved(e);
    }

    pub(crate) fn after_entity_moved(&mut self, e: Entity) { self.do_fov(e); }

    pub(crate) fn entity_step(&mut self, e: Entity, dir: Dir6) -> ActionOutcome {
        if self.confused_move(e) {
            Some(true)
        } else {
            self.really_step(e, dir)
        }
    }

    pub(crate) fn really_step(&mut self, e: Entity, dir: Dir6) -> ActionOutcome {
        let origin = self.location(e)?;
        let loc = origin.jump(self, dir);
        if self.can_enter(e, loc) {
            self.place_entity(e, loc);

            let delay = self.action_delay(e);
            debug_assert!(delay > 0);
            let anim_tick = self.get_anim_tick();
            if let Some(anim) = self.ecs_mut().anim.get_mut(e) {
                anim.tween_from = origin;
                anim.tween_start = anim_tick;
                anim.tween_duration = delay;
            }
            self.end_turn(e);
            return Some(true);
        }

        None
    }

    /// Randomly make a confused mob move erratically.
    ///
    /// Return true if confusion kicked in.
    pub(crate) fn confused_move(&mut self, e: Entity) -> bool {
        const CONFUSE_CHANCE_ONE_IN: u32 = 3;

        if !self.has_status(e, Status::Confused) {
            return false;
        }

        if self.rng().one_chance_in(CONFUSE_CHANCE_ONE_IN) {
            let dir = self.rng().gen();
            let loc = if let Some(loc) = self.location(e) {
                loc
            } else {
                return false;
            };
            let destination = loc.jump(self, dir);

            if self.mob_at(destination).is_some() {
                let _ = self.really_melee(e, dir);
            } else {
                let _ = self.really_step(e, dir);
            }
            true
        } else {
            false
        }
    }

    /// Return whether the entity can move in a direction.
    pub fn can_step(&self, e: Entity, dir: Dir6) -> bool {
        self.location(e)
            .map_or(false, |loc| self.can_enter(e, loc.jump(self, dir)))
    }

    /// Return whether the entity can move in a direction based on just the terrain.
    ///
    /// There might be blocking mobs but they are ignored
    pub fn can_step_on_terrain(&self, e: Entity, dir: Dir6) -> bool {
        self.location(e)
            .map_or(false, |loc| self.can_enter_terrain(e, loc.jump(self, dir)))
    }

    /// Return whether location blocks line of sight.
    pub fn blocks_sight(&self, loc: Location) -> bool { self.terrain(loc).blocks_sight() }

    /// Return whether the entity can occupy a location.
    pub fn can_enter(&self, e: Entity, loc: Location) -> bool {
        if self.terrain(loc).is_door() && !self.has_intrinsic(e, Intrinsic::Hands) {
            // Can't open doors without hands.
            return false;
        }
        if self.blocks_walk(loc) {
            return false;
        }
        true
    }

    pub fn can_enter_terrain(&self, e: Entity, loc: Location) -> bool {
        if self.terrain(loc).is_door() && !self.has_intrinsic(e, Intrinsic::Hands) {
            // Can't open doors without hands.
            return false;
        }
        if self.terrain_blocks_walk(loc) {
            return false;
        }
        true
    }

    /// Return whether the entity blocks movement of other entities.
    pub fn is_blocking_entity(&self, e: Entity) -> bool { self.is_mob(e) }

    /// Return whether the location obstructs entity movement.
    pub fn blocks_walk(&self, loc: Location) -> bool {
        if self.terrain_blocks_walk(loc) {
            return true;
        }
        if self
            .entities_at(loc)
            .into_iter()
            .any(|e| self.is_blocking_entity(e))
        {
            return true;
        }
        false
    }

    /// Return whether the location obstructs entity movement.
    pub fn terrain_blocks_walk(&self, loc: Location) -> bool {
        if !self.is_valid_location(loc) {
            return true;
        }
        if self.terrain(loc).blocks_walk() {
            return true;
        }
        false
    }

    /// Return whether a location contains mobs.
    pub fn has_mobs(&self, loc: Location) -> bool { self.mob_at(loc).is_some() }

    /// Return mob (if any) at given location.
    pub fn mob_at(&self, loc: Location) -> Option<Entity> {
        self.entities_at(loc).into_iter().find(|&e| self.is_mob(e))
    }

    pub fn distance_between(&self, e1: Entity, e2: Entity) -> Option<i32> {
        self.location(e1)?.distance_from(self.location(e2)?)
    }

    pub fn sector_exists(&self, sector: Sector) -> bool { self.world_cache.sector_exists(sector) }

    pub fn is_underground(&self, loc: Location) -> bool { loc.z < 0 }

    pub fn light_level(&self, loc: Location) -> f32 {
        // Lit terrain is lit.
        if self.terrain(loc).is_luminous() {
            return 1.0;
        }

        // In dark arears, far-away things are dim.
        if self.is_underground(loc) {
            if let Some(player) = self.player() {
                if let Some(player_loc) = self.location(player) {
                    // XXX: This is going to get so messed up with portals, should be done in
                    // player chart space, not here...
                    if let Some(dist) = player_loc.distance_from(loc) {
                        return (0.0..=1.0).clamp(1.0 - (dist as f32 / 8.0));
                    }
                }
            }
        }

        // Otherwise things are bright.
        1.0
    }
}
