//! Gameplay logic that answers questions but doesn't change anything

use crate::{
    components::{Alignment, BrainState, Status},
    grammar::{Noun, Pronoun},
    item::{self, EquipType},
    location::Location,
    mapsave,
    spec::EntitySpawn,
    stats::{self, Intrinsic},
    Ability, Ecs, FovStatus, Icon, ItemType, Slot, Terrain, World,
};
use calx::{clamp, hex_neighbors, CellVector, Dir6, HexGeom, Noise};
use calx_ecs::Entity;
use euclid::vec2;
use rand::distributions::Uniform;
use std::collections::{HashSet, VecDeque};
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

    /// Return maximum health of an entity.
    pub fn max_hp(&self, e: Entity) -> i32 { self.stats(e).power }

    /// Return reference to the world entity component system.
    pub fn ecs(&self) -> &Ecs { &self.ecs }

    /// Return the AI state of an entity.
    pub fn brain_state(&self, e: Entity) -> Option<BrainState> {
        self.ecs().brain.get(e).and_then(|brain| Some(brain.state))
    }

    /// Return whether the entity is a mobile object (eg. active creature).
    pub fn is_mob(&self, e: Entity) -> bool { self.ecs().brain.contains(e) }

    pub fn is_item(&self, e: Entity) -> bool { self.ecs().item.contains(e) }

    /// Return the value for how a mob will react to other mobs.
    pub fn alignment(&self, e: Entity) -> Option<Alignment> {
        self.ecs().brain.get(e).map(|b| b.alignment)
    }

    /// Return current health of an entity.
    pub fn hp(&self, e: Entity) -> i32 {
        self.max_hp(e)
            - if self.ecs().health.contains(e) {
                self.ecs().health[e].wounds
            } else {
                0
            }
    }

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

    /// Return visual brush for an entity.
    pub fn entity_icon(&self, e: Entity) -> Option<Icon> { self.ecs().desc.get(e).map(|x| x.icon) }

    pub fn entity_name(&self, e: Entity) -> String {
        if let Some(desc) = self.ecs().desc.get(e) {
            let count = self.count(e);

            if count > 1 {
                format!("{} {}", count, desc.plural_name())
            } else {
                desc.singular_name.clone()
            }
        } else {
            "N/A".to_string()
        }
    }

    pub fn noun(&self, e: Entity) -> Noun {
        let mut ret = Noun::new(self.entity_name(e));
        if self.is_player(e) {
            ret = ret.you().pronoun(Pronoun::They);
        }
        if self.count(e) > 1 {
            ret = ret.plural();
        }
        // TODO: Human mobs get he/she pronoun instead of it.
        ret
    }

    /// Return the (composite) stats for an entity.
    ///
    /// Will return the default value for the Stats type (additive identity in the stat algebra)
    /// for entities that have no stats component defined.
    pub fn stats(&self, e: Entity) -> stats::Stats {
        self.ecs()
            .stats
            .get(e)
            .map(|s| s.actual)
            .unwrap_or_default()
    }

    /// Return the base stats of the entity. Does not include any added effects.
    ///
    /// You usually want to use the `stats` method instead of this one.
    pub fn base_stats(&self, e: Entity) -> stats::Stats {
        self.ecs().stats.get(e).map(|s| s.base).unwrap_or_default()
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

    pub fn can_drop_item_at(&self, loc: Location) -> bool {
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

    /// Return first item at given location.
    pub fn item_at(&self, loc: Location) -> Option<Entity> {
        self.entities_at(loc).into_iter().find(|&e| self.is_item(e))
    }

    /// Return whether the entity has a specific intrinsic property (eg. poison resistance).
    pub fn has_intrinsic(&self, e: Entity, intrinsic: Intrinsic) -> bool {
        self.stats(e).intrinsics & (1 << intrinsic as u32) != 0
    }

    /// Return whether the entity has a specific temporary status
    pub fn has_status(&self, e: Entity, status: Status) -> bool {
        self.ecs()
            .status
            .get(e)
            .map_or(false, |s| s.contains_key(&status))
    }

    /// Return how many frames the entity will delay after an action.
    pub fn action_delay(&self, e: Entity) -> u32 {
        // Granular speed system:
        // | slow and slowed  | 1 |
        // | slow or slowed   | 2 |
        // | normal           | 3 |
        // | quick or hasted  | 4 |
        // | quick and hasted | 5 |

        let mut speed = 3;
        if self.has_intrinsic(e, Intrinsic::Slow) {
            speed -= 1;
        }
        if self.has_status(e, Status::Slowed) {
            speed -= 1;
        }
        if self.has_intrinsic(e, Intrinsic::Quick) {
            speed += 1;
        }
        if self.has_status(e, Status::Hasted) {
            speed += 1;
        }

        match speed {
            1 => 36,
            2 => 18,
            3 => 12,
            4 => 9,
            5 => 7,
            _ => panic!("Invalid speed value {}", speed),
        }
    }

    /// Return if the entity is a mob that should get an update this frame
    /// based on its speed properties. Does not check for status effects like
    /// sleep that might prevent actual action.
    pub fn ticks_this_frame(&self, e: Entity) -> bool {
        if !self.is_mob(e) || !self.is_alive(e) {
            return false;
        }

        if self.has_status(e, Status::Delayed) {
            return false;
        }

        true
    }

    /// Return whether the entity is dead and should be removed from the world.
    pub fn is_alive(&self, e: Entity) -> bool { self.location(e).is_some() }

    /// Return true if the game has ended and the player can make no further
    /// actions.
    pub fn game_over(&self) -> bool { self.player().is_none() }

    /// Return whether an entity is the player avatar mob.
    pub fn is_player(&self, e: Entity) -> bool {
        // TODO: Should this just check self.flags.player?
        self.brain_state(e) == Some(BrainState::PlayerControl) && self.is_alive(e)
    }

    /// Return whether an entity is under computer control
    pub fn is_npc(&self, e: Entity) -> bool { self.is_mob(e) && !self.is_player(e) }

    /// Return whether the entity is an awake mob.
    pub fn is_active(&self, e: Entity) -> bool {
        match self.brain_state(e) {
            Some(BrainState::Asleep) => false,
            Some(_) => true,
            _ => false,
        }
    }

    /// Return whether the entity is a mob that will act this frame.
    pub fn acts_this_frame(&self, e: Entity) -> bool {
        if !self.is_active(e) {
            return false;
        }
        self.ticks_this_frame(e)
    }

    pub fn player_can_act(&self) -> bool {
        if let Some(p) = self.player() {
            self.acts_this_frame(p)
        } else {
            false
        }
    }

    /// Look for targets to shoot in a direction.
    pub fn find_target(&self, shooter: Entity, dir: Dir6, range: usize) -> Option<Entity> {
        let origin = self.location(shooter).unwrap();
        let mut loc = origin;
        for _ in 1..=range {
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
    pub fn pathing_dir_towards(&self, e: Entity, destination: Location) -> Option<Dir6> {
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
    pub fn is_hostile_to(&self, e: Entity, other: Entity) -> bool {
        let (a, b) = (self.alignment(e), self.alignment(other));
        if a.is_none() || b.is_none() {
            return false;
        }

        // Chaotics fight everything, otherwise different alignments fight.
        a == Some(Alignment::Chaotic) || a != b
    }

    /// Return whether the entity should have an idle animation.
    pub fn is_bobbing(&self, e: Entity) -> bool { self.is_active(e) && !self.is_player(e) }

    pub fn item_type(&self, e: Entity) -> Option<ItemType> {
        self.ecs().item.get(e).and_then(|item| Some(item.item_type))
    }

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

    /// Return the name that can be used to spawn this entity.
    pub fn spawn_name(&self, e: Entity) -> Option<&str> {
        // TODO: Create a special component for this.
        self.ecs()
            .desc
            .get(e)
            .and_then(|desc| Some(&desc.singular_name[..]))
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

    pub fn free_bag_slot(&self, e: Entity) -> Option<Slot> {
        (0..item::BAG_CAPACITY)
            .find(|&i| self.entity_equipped(e, Slot::Bag(i)).is_none())
            .map(Slot::Bag)
    }

    pub fn free_equip_slot(&self, e: Entity, item: Entity) -> Option<Slot> {
        Slot::equipment_iter()
            .find(|&&x| x.accepts(self.equip_type(item)) && self.entity_equipped(e, x).is_none())
            .cloned()
    }

    /// Find a drop position for an item, trying to keep one item per cell.
    ///
    /// Dropping several items in the same location will cause them to spread out to the adjacent
    /// cells. If there is no room for the items to spread out, they will be stacked on the initial
    /// drop site.
    pub fn empty_item_drop_location(&self, origin: Location) -> Location {
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

    /// Return the set of mobs that are in update range.
    ///
    /// In a large game world, the active set is limited to the player's surroundings.
    pub fn active_mobs(&self) -> Vec<Entity> {
        self.entities()
            .filter(|&&e| self.is_mob(e))
            .cloned()
            .collect()
    }

    /// Return number of times item can be used.
    pub fn uses_left(&self, item: Entity) -> u32 {
        self.ecs().item.get(item).map_or(0, |i| i.charges)
    }

    pub fn destroy_after_use(&self, item: Entity) -> bool {
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

    pub fn equip_type(&self, item: Entity) -> Option<EquipType> {
        use crate::ItemType::*;
        match self.item_type(item) {
            Some(MeleeWeapon) => Some(EquipType::Melee),
            Some(RangedWeapon) => Some(EquipType::Ranged),
            Some(Helmet) => Some(EquipType::Head),
            Some(Armor) => Some(EquipType::Body),
            Some(Boots) => Some(EquipType::Feet),
            Some(Trinket) => Some(EquipType::Trinket),
            _ => None,
        }
    }

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
                        return clamp(0.0, 1.0, 1.0 - (dist as f32 / 8.0));
                    }
                }
            }
        }

        // Otherwise things are bright.
        1.0
    }

    /// Return count on entity if it's a stack
    pub fn count(&self, e: Entity) -> u32 {
        if let Some(stacking) = self.ecs().stacking.get(e) {
            debug_assert!(stacking.count >= 1, "Invalid item stack size");
            stacking.count
        } else {
            1
        }
    }

    pub fn max_stack_size(&self, e: Entity) -> u32 {
        if self.ecs().stacking.contains(e) {
            99
        } else {
            1
        }
    }

    pub fn has_ability(&self, e: Entity, ability: Ability) -> bool {
        self.list_abilities(e).into_iter().any(|x| x == ability)
    }

    pub fn list_abilities(&self, e: Entity) -> Vec<Ability> {
        // Check for item abilities.
        if let Some(item) = self.ecs().item.get(e) {
            match item.item_type {
                ItemType::UntargetedUsable(ability) => {
                    return vec![ability];
                }
                ItemType::TargetedUsable(ability) => {
                    return vec![ability];
                }
                ItemType::Instant(ability) => {
                    return vec![ability];
                }
                _ => {}
            }
        }

        // Entity has no abilites.
        Vec::new()
    }
}
