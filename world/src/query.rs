use FovStatus;
use Prefab;
use calx_ecs::Entity;
use calx_grid::Dir6;
use components::{Alignment, BrainState, Icon};
use form;
use location::Location;
use stats;
use stats::Intrinsic;
use std::iter::FromIterator;
use std::slice;
use terraform::TerrainQuery;
use terrain::Terrain;
use world::Ecs;

/// Immutable querying of game world state.
pub trait Query: TerrainQuery {
    /// Return the location of an entity.
    ///
    /// Returns the location of the containing entity for entities inside
    /// containers. It is possible for entities to not have a location.
    fn location(&self, e: Entity) -> Option<Location>;

    /// Return the player entity if one exists.
    fn player(&self) -> Option<Entity>;

    /// Return current time of the world logic clock.
    fn tick(&self) -> u64;

    /// Return world RNG seed
    fn rng_seed(&self) -> u32;

    /// Return maximum health of an entity.
    fn max_hp(&self, e: Entity) -> i32 { self.stats(e).power }

    /// Return all entities in the world.
    fn entities(&self) -> slice::Iter<Entity>;

    // XXX: Would be nicer if entities_at returned an iterator. Probably want to wait for impl
    // Trait return types before jumping to this.

    /// Return entities at the given location.
    fn entities_at(&self, loc: Location) -> Vec<Entity>;

    /// Return reference to the world entity component system.
    fn ecs(&self) -> &Ecs;

    /// Return the AI state of an entity.
    fn brain_state(&self, e: Entity) -> Option<BrainState> {
        self.ecs().brain.get(e).map_or(
            None,
            |brain| Some(brain.state),
        )
    }

    /// Return whether the entity is a mobile object (eg. active creature).
    fn is_mob(&self, e: Entity) -> bool { self.ecs().brain.contains(e) }

    /// Return the value for how a mob will react to other mobs.
    fn alignment(&self, e: Entity) -> Option<Alignment> {
        self.ecs().brain.get(e).map(|b| b.alignment)
    }

    /// Return current health of an entity.
    fn hp(&self, e: Entity) -> i32 {
        self.max_hp(e) -
            if self.ecs().health.contains(e) {
                self.ecs().health[e].wounds
            } else {
                0
            }
    }

    /// Return field of view for a location.
    fn fov_status(&self, loc: Location) -> Option<FovStatus> {
        if let Some(p) = self.player() {
            if self.ecs().map_memory.contains(p) {
                if self.ecs().map_memory[p].seen.contains(&loc) {
                    return Some(FovStatus::Seen);
                }
                if self.ecs().map_memory[p].remembered.contains(&loc) {
                    return Some(FovStatus::Remembered);
                }
                return None;
            }
        }
        // Just show everything by default.
        Some(FovStatus::Seen)
    }

    /// Return visual brush for an entity.
    fn entity_icon(&self, e: Entity) -> Option<Icon> { self.ecs().desc.get(e).map(|x| x.icon) }

    /// Return the (composite) stats for an entity.
    ///
    /// Will return the default value for the Stats type (additive identity in the stat algebra)
    /// for entities that have no stats component defined.
    fn stats(&self, e: Entity) -> stats::Stats {
        self.ecs().composite_stats.get(e).map_or_else(
            || self.base_stats(e),
            |x| x.0,
        )
    }

    /// Return the base stats of the entity. Does not include any added effects.
    ///
    /// You usually want to use the `stats` method instead of this one.
    fn base_stats(&self, e: Entity) -> stats::Stats {
        self.ecs().stats.get(e).cloned().unwrap_or_default()
    }

    /// Return whether the entity can move in a direction.
    fn can_step(&self, e: Entity, dir: Dir6) -> bool {
        self.location(e).map_or(false, |loc| {
            self.can_enter(e, loc + dir.to_v2())
        })
    }

    /// Return whether location blocks line of sight.
    fn blocks_sight(&self, loc: Location) -> bool { self.terrain(loc).blocks_sight() }

    /// Return whether the entity can occupy a location.
    fn can_enter(&self, e: Entity, loc: Location) -> bool {
        if self.terrain(loc).is_door() && !self.has_intrinsic(e, Intrinsic::Hands) {
            // Can't open doors without hands.
            return false;
        }
        if self.blocks_walk(loc) {
            return false;
        }
        true
    }

    /// Return whether the entity blocks movement of other entities.
    fn is_blocking_entity(&self, e: Entity) -> bool { self.is_mob(e) }

    /// Return whether the location obstructs entity movement.
    fn blocks_walk(&self, loc: Location) -> bool {
        if !self.is_valid_location(loc) {
            return true;
        }
        if self.terrain(loc).blocks_walk() {
            return true;
        }
        if self.entities_at(loc).into_iter().any(|e| {
            self.is_blocking_entity(e)
        })
        {
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
            0 => true,
            1 => self.has_intrinsic(e, Intrinsic::Fast),
            2 => true,
            3 => self.has_intrinsic(e, Intrinsic::Quick),
            4 => !self.has_intrinsic(e, Intrinsic::Slow),
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
        self.ticks_this_frame(e)
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

    /// Return whether the entity should have an idle animation.
    fn is_bobbing(&self, e: Entity) -> bool { self.is_active(e) && !self.is_player(e) }

    /// Return terrain at location for drawing on screen.
    ///
    /// Terrain is sometimes replaced with a variant for visual effect, but
    /// this should not be reflected in the logical terrain.
    fn visual_terrain(&self, loc: Location) -> Terrain {
        use Terrain::*;

        let mut t = self.terrain(loc);
        // Floor terrain dot means "you can step here". So if the floor is outside the valid play
        // area, don't show the dot.
        //
        // XXX: Hardcoded set of floors, must be updated whenever a new floor type is added.
        if !self.is_valid_location(loc) && (t == Ground || t == Grass || t == Gate) {
            t = Empty;
        }


        // TODO: Might want a more generic method of specifying cosmetic terrain variants.
        if t == Grass && loc.noise() > 0.85 {
            // Grass is occasionally fancy.
            t = Grass2;
        }

        t
    }

    /// Return the name that can be used to spawn this entity.
    fn spawn_name(&self, e: Entity) -> Option<&str> {
        // TODO: Create a special component for this.
        self.ecs().desc.get(e).map_or(
            None,
            |desc| Some(&desc.name[..]),
        )
    }

    fn is_spawn_name(&self, spawn_name: &str) -> bool {
        form::FORMS.iter().any(|f| f.name() == Some(spawn_name))
    }

    fn extract_prefab<I: IntoIterator<Item = Location>>(&self, locs: I) -> Prefab {
        let mut map = Vec::new();
        let mut origin = None;

        for loc in locs {
            // Store first location as an arbitrary origin.
            let origin = match origin {
                None => {
                    origin = Some(loc);
                    loc
                }
                Some(origin) => origin,
            };

            let pos = origin.v2_at(loc).expect(
                "Trying to build prefab from multiple z-levels",
            );

            let terrain = self.terrain(loc);

            let entities = Vec::from_iter(self.entities_at(loc).into_iter().filter_map(|e| {
                self.spawn_name(e).map(|s| s.to_string())
            }));

            map.push((pos, (terrain, entities)));
        }

        Prefab::from_iter(map.into_iter())
    }
}
