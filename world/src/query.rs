//! Gameplay logic that answers questions but doesn't change anything

use crate::{
    fov::SightFov,
    grammar::{Noun, Pronoun},
    item::{self, EquipType},
    location::Location,
    mapsave,
    spec::EntitySpawn,
    stats::Intrinsic,
    Ecs, FovStatus, Icon, ItemType, Sector, Slot, Terrain, World,
};
use calx::{
    hex_neighbors, CellVector, Clamp, Dir6, HexFov, HexFovIter, HexGeom, Noise,
};
use calx_ecs::Entity;
use euclid::vec2;
use indexmap::IndexSet;
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

    /// Return reference to the world entity component system.
    pub fn ecs(&self) -> &Ecs { &self.ecs }

    pub fn is_item(&self, e: Entity) -> bool { self.ecs().item.contains(e) }

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
    pub fn entity_icon(&self, e: Entity) -> Option<Icon> {
        self.ecs().desc.get(e).map(|x| x.icon)
    }

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
    pub fn blocks_sight(&self, loc: Location) -> bool {
        self.terrain(loc).blocks_sight()
    }

    /// Return whether the entity can occupy a location.
    pub fn can_enter(&self, e: Entity, loc: Location) -> bool {
        if self.terrain(loc).is_door()
            && !self.has_intrinsic(e, Intrinsic::Hands)
        {
            // Can't open doors without hands.
            return false;
        }
        if self.blocks_walk(loc) {
            return false;
        }
        true
    }

    pub fn can_enter_terrain(&self, e: Entity, loc: Location) -> bool {
        if self.terrain(loc).is_door()
            && !self.has_intrinsic(e, Intrinsic::Hands)
        {
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

    /// Return true if the game has ended and the player can make no further
    /// actions.
    pub fn game_over(&self) -> bool { self.player().is_none() }

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

    pub fn extract_prefab<I: IntoIterator<Item = Location>>(
        &self,
        locs: I,
    ) -> mapsave::Prefab {
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
            .find(|&&x| {
                x.accepts(self.equip_type(item))
                    && self.entity_equipped(e, x).is_none()
            })
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
            self.can_drop_item_at(origin.jump(self, v))
                && v.hex_dist() <= MAX_SPREAD_DISTANCE
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
                if v.hex_dist() > current_dist
                    && !seen.contains(&v)
                    && is_valid(v)
                {
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
    pub fn projected_explosion_center(
        &self,
        origin: Location,
        dir: Dir6,
        range: u32,
    ) -> Location {
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
                        return (0.0..=1.0).clamp(1.0 - (dist as f32 / 8.0));
                    }
                }
            }
        }

        // Otherwise things are bright.
        1.0
    }

    pub fn sector_exists(&self, sector: Sector) -> bool {
        self.world_cache.sector_exists(sector)
    }

    pub fn fov_from(&self, origin: Location, range: i32) -> IndexSet<Location> {
        // Use IndexSet as return type because eg. AI logic for dealing with seen things may depend
        // on iteration order.
        debug_assert!(range >= 0);

        IndexSet::from_iter(
            HexFov::new(SightFov::new(self, range as u32, origin))
                .add_fake_isometric_acute_corners(|pos, a| {
                    self.terrain(a.origin + pos).is_wall()
                })
                .map(|(pos, a)| a.origin + pos),
        )
    }

    pub fn distance_between(&self, e1: Entity, e2: Entity) -> Option<i32> {
        self.location(e1)?.distance_from(self.location(e2)?)
    }
}
