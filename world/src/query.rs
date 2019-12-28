//! Gameplay logic that answers questions but doesn't change anything

use crate::{
    fov::SightFov, location::Location, mapsave, spec::EntitySpawn, Ecs, FovStatus, Terrain, World,
};
use calx::{Dir6, HexFov, HexFovIter, Noise};
use calx_ecs::Entity;
use indexmap::IndexSet;
use rand::distributions::Uniform;
use std::iter::FromIterator;
use std::str::FromStr;

impl World {
    /// Return the player entity if one exists.
    pub fn player(&self) -> Option<Entity> {
        if let Some(p) = self.flags.player {
            if self.is_alive(p) {
                return Some(p);
            }
        }

        None
    }

    /// Return current time of the world logic clock.
    pub fn get_tick(&self) -> u64 { self.flags.tick }

    /// Return world RNG seed
    pub fn rng_seed(&self) -> u32 { self.world_cache.seed() }

    /// Return reference to the world entity component system.
    pub fn ecs(&self) -> &Ecs { &self.ecs }

    /// Return field of view for a location.
    pub fn fov_status(&self, loc: Location) -> Option<FovStatus> {
        if let Some(p) = self.player() {
            if self.ecs().map_memory.contains(p) {
                if self.ecs().map_memory[p].seen.contains(loc) {
                    return Some(FovStatus::Seen);
                }
                if self.ecs().map_memory[p].remembered.contains(loc) {
                    return Some(FovStatus::Remembered);
                }
                return None;
            }
        }
        // Just show everything by default.
        Some(FovStatus::Seen)
    }

    /// Return true if the game has ended and the player can make no further
    /// actions.
    pub fn game_over(&self) -> bool { self.player().is_none() }

    /// Return terrain at location for drawing on screen.
    ///
    /// Terrain is sometimes replaced with a variant for visual effect, but
    /// this should not be reflected in the logical terrain.
    pub fn visual_terrain(&self, loc: Location) -> Terrain {
        use crate::Terrain::*;

        let mut t = self.terrain(loc);

        // Draw gates under portals when drawing non-portaled stuff
        if t == Empty && self.portal(loc).is_some() {
            return Downstairs;
        }

        // Floor terrain dot means "you can step here". So if the floor is outside the valid play
        // area, don't show the dot.
        //
        // XXX: Hardcoded set of floors, must be updated whenever a new floor type is added.
        if !self.is_valid_location(loc)
            && (t == Ground || t == Grass || t == Downstairs || t == Upstairs)
        {
            t = Empty;
        }

        // TODO: Might want a more generic method of specifying cosmetic terrain variants.
        if t == Grass && Uniform::new_inclusive(0.0, 1.0).noise(&loc) > 0.95 {
            // Grass is occasionally fancy.
            t = Grass2;
        }

        t
    }

    pub fn extract_prefab<I: IntoIterator<Item = Location>>(&self, locs: I) -> mapsave::Prefab {
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

            let pos = origin
                .v2_at(loc)
                .expect("Trying to build prefab from multiple z-levels");

            let terrain = self.terrain(loc);

            let entities: Vec<_> = self
                .entities_at(loc)
                .into_iter()
                .filter_map(|e| self.spawn_name(e))
                .map(|n| EntitySpawn::from_str(n).unwrap())
                .collect();

            map.push((pos, (terrain, entities)));
        }

        mapsave::Prefab::from_iter(map.into_iter())
    }

    /// Find a location for spell explosion.
    ///
    /// Explosion centers will penetrate and hit cells with mobs, they will stop before cells with
    /// blocking terrain.
    pub fn projected_explosion_center(&self, origin: Location, dir: Dir6, range: u32) -> Location {
        let mut loc = origin;
        for _ in 0..range {
            let new_loc = loc.jump(self, dir);

            if self.has_mobs(new_loc) {
                return new_loc;
            }

            if self.terrain(new_loc).blocks_shot() {
                return loc;
            }

            loc = new_loc;
        }
        loc
    }

    /// Return whether the player can currently directly see the given location.
    pub fn player_sees(&self, loc: Location) -> bool {
        self.fov_status(loc) == Some(FovStatus::Seen)
    }

    pub fn fov_from(&self, origin: Location, range: i32) -> IndexSet<Location> {
        // Use IndexSet as return type because eg. AI logic for dealing with seen things may depend
        // on iteration order.
        debug_assert!(range >= 0);

        IndexSet::from_iter(
            HexFov::new(SightFov::new(self, range as u32, origin))
                .add_fake_isometric_acute_corners(|pos, a| self.terrain(a.origin + pos).is_wall())
                .map(|(pos, a)| a.origin + pos),
        )
    }
}
