use std::collections::HashSet;
use std::iter::FromIterator;
use std::rc::Rc;
use calx_grid::{Dir6, HexFov};
use calx_ecs::Entity;
use calx_resource::{Resource, ResourceStore};
use world::World;
use components::{Alignment, BrainState};
use stats::{Intrinsic, Stats};
use location::Location;
use spatial::Place;
use brush::Brush;
use terrain;
use FovStatus;
use fov::SightFov;

/// Immutable querying of game world state.
pub trait Query {
    /// Return the location of an entity.
    ///
    /// Returns the location of the containing entity for entities inside
    /// containers. It is possible for entities to not have a location.
    fn location(&self, e: Entity) -> Option<Location>;

    /// Return the player entity if one exists.
    fn player(&self) -> Option<Entity>;

    /// Return the AI state of an entity.
    fn brain_state(&self, e: Entity) -> Option<BrainState>;

    /// Return current time of the world logic clock.
    fn tick(&self) -> u64;

    /// Return whether the entity is a mobile object (eg. active creature).
    fn is_mob(&self, e: Entity) -> bool;

    /// Return the value for how a mob will react to other mobs.
    fn alignment(&self, e: Entity) -> Option<Alignment>;

    /// Return terrain at location.
    fn terrain(&self, loc: Location) -> Rc<terrain::Tile>;

    /// If location contains a portal, return the destination of the portal.
    fn portal(&self, loc: Location) -> Option<Location>;

    /// Return whether the entity can move in a direction.
    fn can_step(&self, e: Entity, dir: Dir6) -> bool {
        self.location(e).map_or(false, |loc| self.can_enter(e, loc + dir.to_v2()))
    }

    /// Return current health of an entity.
    fn hp(&self, e: Entity) -> i32;

    /// Return maximum health of an entity.
    fn max_hp(&self, e: Entity) -> i32 {
        self.stats(e).power
    }

    /// Return field of view for a location.
    fn fov_status(&self, loc: Location) -> Option<FovStatus>;

    /// Return visual brush for an entity.
    fn entity_brush(&self, e: Entity) -> Option<Resource<Brush>>;

    // XXX: Would be nicer if entities_at returned an iterator. Probably want to wait for impl
    // Trait return types before jumping to this.

    /// Return an iterator to entities at the given location.
    fn entities_at(&self, loc: Location) -> Vec<Entity>;

    /// Return whether location blocks line of sight.
    fn blocks_sight(&self, loc: Location) -> bool { self.terrain(loc).blocks_sight() }

    /// Return a portal if it can be seen through.
    fn visible_portal(&self, loc: Location) -> Option<Location> {
        // Only void-form is transparent to portals.
        if self.terrain(loc).form == terrain::Form::Void { self.portal(loc) } else { None }
    }

    /// Return whether the entity can occupy a location.
    fn can_enter(&self, e: Entity, loc: Location) -> bool {
        if self.is_mob(e) && self.has_mobs(loc) {
            // Can only have one mob per cell.
            return false;
        }
        if self.terrain(loc).is_door() && !self.has_intrinsic(e, Intrinsic::Hands) {
            // Can't open doors without hands.
            return false;
        }
        if self.blocks_walk(loc) {
            return false;
        }
        true
    }

    /// Return the (composite) stats for an entity.
    ///
    /// Will return the default value for the Stats type (additive identity in the stat algebra)
    /// for entities that have no stats component defined.
    fn stats(&self, e: Entity) -> Stats;

    /// Return whether the entity blocks movement of other entities.
    fn is_blocking_entity(&self, e: Entity) -> bool { self.is_mob(e) }

    /// Return whether the location obstructs entity movement.
    fn blocks_walk(&self, loc: Location) -> bool {
        if self.terrain(loc).blocks_walk() {
            return true;
        }
        if self.entities_at(loc)
                .into_iter()
                .any(|e| self.is_blocking_entity(e)) {
                    return true;
                }
        false
    }

    /// Return whether a location contains mobs.
    fn has_mobs(&self, loc: Location) -> bool { self.mob_at(loc).is_some() }

    /// Return mob (if any) at given position.
    fn mob_at(&self, loc: Location) -> Option<Entity> {
        self.entities_at(loc).into_iter().find(|&e| self.is_mob(e))
    }

    /// Return the base stats of the entity. Does not include any added effects.
    ///
    /// You usually want to use the `stats` method instead of this one.
    fn base_stats(&self, e: Entity) -> Stats;

    /// Return whether the entity has a specific intrinsic property (eg. poison resistance).
    fn has_intrinsic(&self, e: Entity, intrinsic: Intrinsic) -> bool {
        self.stats(e).intrinsics & intrinsic as u32 != 0
    }

    /// Return if the entity is a mob that should get an update this frame
    /// based on its speed properties. Does not check for status effects like
    /// sleep that might prevent actual action.
    fn ticks_this_frame(&self, e: Entity) -> bool {
        if !self.is_mob(e) || !self.is_alive(e) {
            return false;
        }

        // Go through a cycle of 5 phases to get 4 possible speeds.
        // System idea from Jeff Lait.
        let phase = self.tick() % 5;
        match phase {
            0 => return true,
            1 => return self.has_intrinsic(e, Intrinsic::Fast),
            2 => return true,
            3 => return self.has_intrinsic(e, Intrinsic::Quick),
            4 => return !self.has_intrinsic(e, Intrinsic::Slow),
            _ => panic!("Invalid action phase"),
        }
    }

    /// Return whether the entity is dead and should be removed from the world.
    fn is_alive(&self, e: Entity) -> bool { self.location(e).is_some() }

    /// Return true if the game has ended and the player can make no further
    /// actions.
    fn game_over(&self) -> bool { self.player().is_none() }

    /// Return whether an entity is the player avatar mob.
    fn is_player(&self, e: Entity) -> bool {
        // TODO: Should this just check self.flags.player?
        self.brain_state(e) == Some(BrainState::PlayerControl) && self.is_alive(e)
    }

    /// Return whether the entity is an awake mob.
    fn is_active(&self, e: Entity) -> bool {
        match self.brain_state(e) {
            Some(BrainState::Asleep) => false,
            Some(_) => true,
            _ => false,
        }
    }

    /// Return whether the entity is a mob that will act this frame.
    fn acts_this_frame(&self, e: Entity) -> bool {
        if !self.is_active(e) {
            return false;
        }
        return self.ticks_this_frame(e);
    }

    /// Look for targets to shoot in a direction.
    fn find_target(&self, shooter: Entity, dir: Dir6, range: usize) -> Option<Entity> {
        let origin = self.location(shooter).unwrap();
        for i in 1..(range + 1) {
            let loc = origin + dir.to_v2() * (i as i32);
            if self.terrain(loc).blocks_shot() {
                break;
            }
            if let Some(e) = self.mob_at(loc) {
                if self.is_hostile_to(shooter, e) {
                    return Some(e);
                }
            }
        }
        None
    }

    /// Return whether the entity wants to fight the other entity.
    fn is_hostile_to(&self, e: Entity, other: Entity) -> bool {
        match (self.alignment(e), self.alignment(other)) {
            (Some(Alignment::Chaotic), Some(_)) => true,
            (Some(_), Some(Alignment::Chaotic)) => true,
            (Some(Alignment::Evil), Some(Alignment::Good)) => true,
            (Some(Alignment::Good), Some(Alignment::Evil)) => true,
            _ => false,
        }
    }

    /// Return whether the entity is an awake non-player mob and should be
    /// animated with a bob.
    fn is_bobbing(&self, e: Entity) -> bool { self.is_active(e) && !self.is_player(e) }

    /// Return the field of view chart for visible tiles.
    fn sight_fov(w: &World, origin: Location, range: u32) -> HashSet<Location> {
        HashSet::from_iter(HexFov::new(SightFov::new(w, range, origin)).map(|(pos, a)| a.origin + pos))
    }
}
