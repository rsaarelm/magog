use FovStatus;
use Prefab;
use calx::{clamp, hex_neighbors, CellVector, Dir6, HexGeom};
use calx_ecs::Entity;
use components::{Alignment, BrainState, Icon, Status};
use euclid::vec2;
use form;
use grammar::{Noun, Pronoun};
use item::{EquipType, ItemType, Slot};
use location::Location;
use stats;
use stats::Intrinsic;
use std::collections::{HashSet, VecDeque};
use std::iter::FromIterator;
use std::slice;
use terraform::TerrainQuery;
use terrain::Terrain;
use volume::Volume;
use world::Ecs;

/// Immutable querying of game world state.
pub trait Query: TerrainQuery + Sized {
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

    /// Return entities inside another entity.
    fn entities_in(&self, parent: Entity) -> Vec<Entity>;

    /// Return reference to the world entity component system.
    fn ecs(&self) -> &Ecs;

    /// Return the item parent has equipped in slot.
    fn entity_equipped(&self, parent: Entity, slot: Slot) -> Option<Entity>;

    fn entity_contains(&self, parent: Entity, child: Entity) -> bool;

    fn sphere_volume(&self, origin: Location, radius: u32) -> Volume;

    /// Return the AI state of an entity.
    fn brain_state(&self, e: Entity) -> Option<BrainState> {
        self.ecs().brain.get(e).and_then(|brain| Some(brain.state))
    }

    /// Return whether the entity is a mobile object (eg. active creature).
    fn is_mob(&self, e: Entity) -> bool { self.ecs().brain.contains(e) }

    fn is_item(&self, e: Entity) -> bool { self.ecs().item.contains(e) }

    /// Return the value for how a mob will react to other mobs.
    fn alignment(&self, e: Entity) -> Option<Alignment> {
        self.ecs().brain.get(e).map(|b| b.alignment)
    }

    /// Return current health of an entity.
    fn hp(&self, e: Entity) -> i32 {
        self.max_hp(e) - if self.ecs().health.contains(e) {
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

    fn entity_name(&self, e: Entity) -> String {
        self.ecs()
            .desc
            .get(e)
            .map_or_else(|| "N/A".to_string(), |x| x.name.clone())
    }

    fn noun(&self, e: Entity) -> Noun {
        let mut ret = Noun::new(self.entity_name(e));
        if self.is_player(e) {
            ret = ret.you().pronoun(Pronoun::They);
        }
        // TODO: Human mobs get he/she pronoun instead of it.
        ret
    }

    /// Return the (composite) stats for an entity.
    ///
    /// Will return the default value for the Stats type (additive identity in the stat algebra)
    /// for entities that have no stats component defined.
    fn stats(&self, e: Entity) -> stats::Stats {
        self.ecs()
            .stats
            .get(e)
            .map(|s| s.actual)
            .unwrap_or_default()
    }

    /// Return the base stats of the entity. Does not include any added effects.
    ///
    /// You usually want to use the `stats` method instead of this one.
    fn base_stats(&self, e: Entity) -> stats::Stats {
        self.ecs().stats.get(e).map(|s| s.base).unwrap_or_default()
    }

    /// Return whether the entity can move in a direction.
    fn can_step(&self, e: Entity, dir: Dir6) -> bool {
        self.location(e)
            .map_or(false, |loc| self.can_enter(e, loc.jump(self, dir)))
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

    fn can_drop_item_at(&self, loc: Location) -> bool {
        if !self.is_valid_location(loc) {
            return false;
        }
        if self.terrain(loc).blocks_walk() {
            return false;
        }
        if self.terrain(loc).is_door() {
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
        if self.entities_at(loc)
            .into_iter()
            .any(|e| self.is_blocking_entity(e))
        {
            return true;
        }
        false
    }

    /// Return whether a location contains mobs.
    fn has_mobs(&self, loc: Location) -> bool { self.mob_at(loc).is_some() }

    /// Return mob (if any) at given location.
    fn mob_at(&self, loc: Location) -> Option<Entity> {
        self.entities_at(loc).into_iter().find(|&e| self.is_mob(e))
    }

    /// Return first item at given location.
    fn item_at(&self, loc: Location) -> Option<Entity> {
        self.entities_at(loc).into_iter().find(|&e| self.is_item(e))
    }

    /// Return whether the entity has a specific intrinsic property (eg. poison resistance).
    fn has_intrinsic(&self, e: Entity, intrinsic: Intrinsic) -> bool {
        self.stats(e).intrinsics & (1 << intrinsic as u32) != 0
    }

    /// Return whether the entity has a specific temporary status
    fn has_status(&self, e: Entity, status: Status) -> bool {
        self.ecs()
            .status
            .get(e)
            .map_or(false, |s| s.contains_key(&status))
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
            1 => self.has_status(e, Status::Fast),
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

    /// Return whether an entity is under computer control
    fn is_npc(&self, e: Entity) -> bool { self.is_mob(e) && !self.is_player(e) }

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
        let mut loc = origin;
        for _ in 1..(range + 1) {
            loc = loc.jump(self, dir);
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

    /// Try to get the next step on the path from origin towards destination.
    ///
    /// Tries to be fast, not necessarily doing proper pathfinding.
    fn pathing_dir_towards(&self, e: Entity, destination: Location) -> Option<Dir6> {
        // Could do all sorts of cool things here eventually like a Dijkstra map cache, but for now
        // just doing very simple stuff.
        if let Some(origin) = self.location(e) {
            if let Some(dir) = origin.dir6_towards(destination) {
                // Try direct approach, the the other directions.
                for &turn in &[0, 1, -1, 2, -2, 3] {
                    let dir = dir + turn;
                    let next_loc = origin.jump(self, dir);
                    if self.can_enter(e, next_loc) {
                        return Some(dir);
                    }
                }
                return None;
            }
        }
        None
    }

    /// Return whether the entity wants to fight the other entity.
    fn is_hostile_to(&self, e: Entity, other: Entity) -> bool {
        let (a, b) = (self.alignment(e), self.alignment(other));
        if a.is_none() || b.is_none() {
            return false;
        }

        // Chaotics fight everything, otherwise different alignments fight.
        a == Some(Alignment::Chaotic) || a != b
    }

    /// Return whether the entity should have an idle animation.
    fn is_bobbing(&self, e: Entity) -> bool { self.is_active(e) && !self.is_player(e) }

    fn item_type(&self, e: Entity) -> Option<ItemType> {
        self.ecs().item.get(e).and_then(|item| Some(item.item_type))
    }

    /// Return terrain at location for drawing on screen.
    ///
    /// Terrain is sometimes replaced with a variant for visual effect, but
    /// this should not be reflected in the logical terrain.
    fn visual_terrain(&self, loc: Location) -> Terrain {
        use Terrain::*;

        let mut t = self.terrain(loc);

        // Draw gates under portals when drawing non-portaled stuff
        if t == Empty && self.portal(loc).is_some() {
            return Gate;
        }

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
        self.ecs().desc.get(e).and_then(|desc| Some(&desc.name[..]))
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

            let pos = origin
                .v2_at(loc)
                .expect("Trying to build prefab from multiple z-levels");

            let terrain = self.terrain(loc);

            let entities = Vec::from_iter(
                self.entities_at(loc)
                    .into_iter()
                    .filter_map(|e| self.spawn_name(e).map(|s| s.to_string())),
            );

            map.push((pos, (terrain, entities)));
        }

        Prefab::from_iter(map.into_iter())
    }

    fn free_bag_slot(&self, e: Entity) -> Option<Slot> {
        Slot::iter()
            .find(|&&x| !x.is_equipment_slot() && self.entity_equipped(e, x).is_none())
            .cloned()
    }

    fn free_equip_slot(&self, e: Entity, item: Entity) -> Option<Slot> {
        if let Some(equip_type) = self.equip_type(item) {
            Slot::iter()
                .find(|&&x| x.accepts(equip_type) && self.entity_equipped(e, x).is_none())
                .cloned()
        } else {
            None
        }
    }

    /// Find a drop position for an item, trying to keep one item per cell.
    ///
    /// Dropping several items in the same location will cause them to spread out to the adjacent
    /// cells. If there is no room for the items to spread out, they will be stacked on the initial
    /// drop site.
    fn empty_item_drop_location(&self, origin: Location) -> Location {
        static MAX_SPREAD_DISTANCE: i32 = 8;
        let is_valid = |v: CellVector| {
            self.can_drop_item_at(origin.jump(self, v)) && v.hex_dist() <= MAX_SPREAD_DISTANCE
        };
        let mut seen = HashSet::new();
        let mut incoming = VecDeque::new();
        incoming.push_back(vec2(0, 0));

        while let Some(offset) = incoming.pop_front() {
            if seen.contains(&offset) {
                continue;
            }

            seen.insert(offset);

            let loc = origin.jump(self, offset);
            if self.item_at(loc).is_none() {
                return loc;
            }

            let current_dist = offset.hex_dist();
            for v in hex_neighbors(offset) {
                if v.hex_dist() > current_dist && !seen.contains(&v) && is_valid(v) {
                    incoming.push_back(v);
                }
            }
        }

        origin
    }

    /// Find a location for spell explosion.
    ///
    /// Explosion centers will penetrate and hit cells with mobs, they will stop before cells with
    /// blocking terrain.
    fn projected_explosion_center(&self, origin: Location, dir: Dir6, range: u32) -> Location {
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
    fn player_sees(&self, loc: Location) -> bool { self.fov_status(loc) == Some(FovStatus::Seen) }

    /// Return the set of mobs that are in update range.
    ///
    /// In a large game world, the active set is limited to the player's surroundings.
    fn active_mobs(&self) -> Vec<Entity> {
        self.entities()
            .filter(|&&e| self.is_mob(e))
            .cloned()
            .collect()
    }

    /// Return number of times item can be used.
    fn uses_left(&self, item: Entity) -> u32 { self.ecs().item.get(item).map_or(0, |i| i.charges) }

    fn destroy_after_use(&self, item: Entity) -> bool {
        // XXX: Fragile. What we want here is to tag potions and scrolls as destroyed when used and
        // wands to stick around. Current item data doesn't have is_potion or is_scroll, but
        // coincidentally the scrolls tend to be untargeted and the wands tend to be targeted
        // spells, so we'll just use that as proxy.
        self.ecs().item.get(item).map_or(false, |i| {
            if let ItemType::UntargetedUsable(_) = i.item_type {
                true
            } else {
                false
            }
        })
    }

    fn equip_type(&self, item: Entity) -> Option<EquipType> {
        use ItemType::*;
        match self.item_type(item) {
            Some(MeleeWeapon) => Some(EquipType::Melee),
            Some(RangedWeapon) => Some(EquipType::Ranged),
            Some(Helmet) => Some(EquipType::Head),
            Some(Armor) => Some(EquipType::Body),
            Some(Boots) => Some(EquipType::Feet),
            Some(Spell) => Some(EquipType::Spell),
            Some(Trinket) => Some(EquipType::Trinket),
            _ => None,
        }
    }

    fn is_underground(&self, loc: Location) -> bool { loc.z > 0 }

    fn light_level(&self, loc: Location) -> f32 {
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
                        return clamp(0.0, 1.0, 1.0 - (dist as f32 / 8.0));
                    }
                }
            }
        }

        // Otherwise things are bright.
        1.0
    }
}
