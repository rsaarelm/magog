use std::default::Default;
use rand::Rng;
use util::Dijkstra;
use util::Rgb;
use world;
use location::{Location};
use dir6::Dir6;
use flags;
use stats::{Intrinsic};
use components::{BrainState, Alignment};
use geom::HexGeom;
use spatial::Place;
use action;
use fov::Fov;
use rng;
use msg;
use item::{ItemType, Slot};
use stats::Stats;
use ecs::{ComponentAccess};

/// Game object handle.
#[derive(Copy, PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Debug, RustcDecodable, RustcEncodable)]
pub struct Entity(pub usize);

impl Entity {
    /// Place the entity in a location in the game world.
    pub fn place(self, loc: Location) {
        assert!(!self.is_prototype(), "Tried to field a prototype");
        world::with_mut(|w| w.spatial.insert_at(self, loc));
        self.on_move_to(loc);
    }

    /// Remove the entity from all the game world systems. THE ENTITY VALUE
    /// WILL BE INVALID FROM HERE ON AND USING IT WILL LEAD TO BUGS. THE
    /// CALLER IS RESPONSIBLE FOR ENSUING THAT AN ENTITY WILL NOT BE
    /// USED FROM ANYWHERE AFTER THE DELETE OPERATION.
    pub fn delete(self) {
        world::with_mut(|w| w.comps.remove(self));
        world::with_mut(|w| w.spatial.remove(self));
        world::with_mut(|w| w.ecs.delete(self));
    }

    pub fn blocks_walk(self) -> bool { self.is_mob() }

    pub fn name(self) -> String {
        world::with(|w|
            match w.descs().get(self) {
                Some(desc) => desc.name.clone(),
                None => "".to_string()
            }
        )
    }

    pub fn get_icon(self) -> Option<(usize, Rgb)> {
        world::with(|w|
            if let Some(desc) = w.descs().get(self) {
                Some((desc.icon, desc.color))
            } else {
                None
            }
        )
    }

    /// Create a cloned entity that uses the current entity as a parent
    /// prototype. Components not defined in the clone entity will be read
    /// from the current entity.
    pub fn clone_at(self, loc: Location) -> Entity {
        let ret = world::with_mut(|w| { w.ecs.new_entity(Some(self)) });
        ret.place(loc);
        ret
    }

    pub fn is_prototype(self) -> bool {
        world::with(|w| w.spawns().get_local(self).is_some())
    }

// Spatial methods /////////////////////////////////////////////////////

    pub fn can_enter(self, loc: Location) -> bool {
        if self.is_mob() && loc.has_mobs() { return false; }
        if loc.terrain().is_door() && !self.has_intrinsic(Intrinsic::Hands) {
            // Can't open doors without hands.
            return false;
        }
        if loc.blocks_walk() { return false; }
        true
    }

    /// Return whether the entity can move in a direction.
    pub fn can_step(self, dir: Dir6) -> bool {
        let place = world::with(|w| w.spatial.get(self));
        if let Some(Place::At(loc)) = place {
            let new_loc = loc + dir.to_v2();
            return self.can_enter(new_loc);
        }
        return false;
    }

    /// Try to move the entity in direction.
    pub fn step(self, dir: Dir6) {
        let place = world::with(|w| w.spatial.get(self));
        if let Some(Place::At(loc)) = place {
            let new_loc = loc + dir.to_v2();
            if self.can_enter(new_loc) {
                world::with_mut(|w| w.spatial.insert_at(self, new_loc));
                self.on_move_to(new_loc);
            }
        }
    }

    pub fn location(self) -> Option<Location> {
        match world::with(|w| w.spatial.get(self)) {
            Some(Place::At(loc)) => Some(loc),
            Some(Place::In(e, _)) => e.location(),
            _ => None
        }
    }

    pub fn distance_from(self, other: Entity) -> Option<i32> {
        if let (Some(loc1), Some(loc2)) = (self.location(), other.location()) {
            loc1.distance_from(loc2)
        } else {
            None
        }
    }

// Damage and lifetime /////////////////////////////////////////////////

    /// Apply damage to entity, subject to damage reduction.
    pub fn damage(self, mut power: i32) {
        let stats = self.stats();
        power -= stats.protection;

        if power < 1 {
            // Give damage a bit under the reduction an off-chance to hit.
            // Power that's too low can't do anything though.
            if power >= -5 { power = 1; } else { return; }
        }

        // Every five points of power is one certain hit.
        let full = power / 5;
        // The fractional points are one probabilistic hit.
        let partial = (power % 5) as f64 / 5.0;

        let damage = full + if rng::p(partial) { 1 } else { 0 };
        self.apply_damage(damage)
    }

    /// Actually subtract points from the entity's hit points. Called from
    /// damage method.
    fn apply_damage(self, amount: i32) {
        if amount <= 0 { return; }
        let max_hp = self.max_hp();

        let (_amount, kill) = world::with_mut(|w| {
            let health = w.healths_mut().get(self).expect("no health");
            health.wounds += amount;
            (amount, health.wounds >= max_hp)
        });

        msg::push(::Msg::Damage(self));
        if kill {
            self.kill();
        }
    }

    pub fn heal(self, amount: i32) {
        if amount <= 0 { return; }
        world::with_mut(|w| {
            let health = w.healths_mut().get(self).expect("no health");
            health.wounds -= amount;
            if health.wounds < 0 {
                health.wounds = 0;
            }
        })
    }

    /// Do any game logic stuff related to this entity dying violently before
    /// deleting it.
    pub fn kill(self) {
        let loc = self.location().expect("no location");
        msgln!("{} dies.", self.name());
        msg::push(::Msg::Gib(loc));
        if rng::one_chance_in(6) {
            // Drop a heart.
            action::spawn_named("heart", loc);
        }
        self.delete();
    }

// Mob methods /////////////////////////////////////////////////////////

    pub fn is_mob(self) -> bool {
        world::with(|w| w.brains().get(self).is_some()) && self.location().is_some()
    }

    /// Return whether this mob is the player avatar.
    pub fn is_player(self) -> bool {
        self.brain_state() == Some(BrainState::PlayerControl) && self.location().is_some()
    }

    /// Return whether this entity is an awake mob.
    pub fn is_active(self) -> bool {
        match self.brain_state() {
            Some(BrainState::Asleep) => false,
            Some(_) => true,
            _ => false
        }
    }

    /// Return if the entity is a mob that should get an update this frame
    /// based on its speed properties. Does not check for status effects like
    /// sleep that might prevent actual action.
    pub fn ticks_this_frame(self) -> bool {
        if self.is_player() { return true; }
        if !self.is_mob() { return false; }

        let tick = flags::get_tick();
        // Go through a cycle of 5 phases to get 4 possible speeds.
        // System idea from Jeff Lait.
        let phase = tick % 5;
        match phase {
            0 => return true,
            1 => return self.has_intrinsic(Intrinsic::Fast),
            2 => return true,
            3 => return false,//self.has_status(Status::Quick),
            4 => return !self.has_intrinsic(Intrinsic::Slow)
                        /*&& !self.has_status(Status::Slow)*/,
            _ => panic!("Invalid action phase"),
        }
    }

    /// Return whether the entity is a mob that will act this frame.
    pub fn acts_this_frame(self) -> bool {
        if !self.is_active() { return false; }
        return self.ticks_this_frame();
    }

    /// Return whether the entity is an awake non-player mob and should be
    /// animated with a bob.
    pub fn is_bobbing(self) -> bool {
        self.is_active() && !self.is_player()
    }

    pub fn melee(self, dir: Dir6) {
        let loc = self.location().expect("no location") + dir.to_v2();
        if let Some(e) = loc.mob_at() {
            let us = self.stats();
            e.damage(us.power + us.attack);
        }
    }

    pub fn hp(self) -> i32 {
        self.max_hp() - world::with(|w|
            if let Some(health) = w.healths().get(self) {
                health.wounds
            } else {
                0
            })
    }

    pub fn max_hp(self) -> i32 {
        self.power_level()
    }

    pub fn is_wounded(self) -> bool {
        world::with(|w|
            if let Some(health) = w.healths().get(self) {
                health.wounds > 0
            } else {
                false
            }
        )
    }

// Invetory methods ////////////////////////////////////////////////////

    /// Return whether entity contains another.
    pub fn contains(self, item: Entity) -> bool {
        world::with(|w| w.spatial.contains(self, item))
    }

    /// Return the item equipped by this entity in the given inventory slot.
    pub fn equipped(self, slot: Slot) -> Option<Entity> {
        world::with(|w| w.spatial.entity_equipped(self, slot))
    }

    /// Equip an item to a slot. Slot must be empty.
    pub fn equip(self, item: Entity, slot: Slot) {
        world::with_mut(|w| w.spatial.equip(item, self, slot));
        self.dirty_stats_cache();
    }

    /// Swap items in two equipment slots
    pub fn swap_equipped(self, slot1: Slot, slot2: Slot) {
        world::with_mut(|w| {
            let eo1 = w.spatial.entity_equipped(self, slot1);
            let eo2 = w.spatial.entity_equipped(self, slot2);
            match (eo1, eo2) {
                (Some(e1), Some(e2)) => {
                    // Temporarily shunt one off to non-slotted containment
                    // space for the switcheroo.
                    w.spatial.insert_in(self, e1);
                    w.spatial.equip(e2, self, slot1);
                    w.spatial.equip(e1, self, slot2);
                }
                (None, Some(e2)) => {
                    w.spatial.equip(e2, self, slot1);
                }
                (Some(e1), None) => {
                    w.spatial.equip(e1, self, slot2);
                }
                _ => {}
            }
        });
        self.dirty_stats_cache();
    }

    pub fn pick_up(self, item: Entity) -> bool {
        if !item.can_be_picked_up() {
            return false;
        }

        match self.free_bag_slot() {
            Some(slot) => {
                self.equip(item, slot);
                return true;
            }
            // Inventory full.
            None => { return false; }
        }
    }

    /// Return the first free storage bag inventory slot on this entity.
    pub fn free_bag_slot(self) -> Option<Slot> {
        for &slot in vec![
            Slot::InventoryJ,
            Slot::InventoryK,
            Slot::InventoryL,
            Slot::InventoryM,
            Slot::InventoryN,
            Slot::InventoryO,
            Slot::InventoryP,
            Slot::InventoryQ,
            Slot::InventoryR,
            Slot::InventoryS,
            Slot::InventoryT,
            Slot::InventoryU,
            Slot::InventoryV,
            Slot::InventoryW,
            Slot::InventoryX,
            Slot::InventoryY,
            Slot::InventoryZ].iter() {
            if self.equipped(slot).is_none() {
                return Some(slot);
            }
        }
        None
    }

// Stats methods ///////////////////////////////////////////////////////

    pub fn has_intrinsic(self, intrinsic: Intrinsic) -> bool {
        self.refresh_stats_cache();
        world::with(|w|
            if let Some(&Some(ref stat)) = w.stats_caches().get(self) {
                stat.intrinsics & intrinsic as u32 != 0
            } else {
                false
            })
    }

    pub fn power_level(self) -> i32 {
        self.refresh_stats_cache();
        world::with(|w|
            if let Some(&Some(ref stat)) = w.stats_caches().get(self) { stat.power }
            else { 0 })
    }

    /// Return the stats structure for the entity. Stats-less entities return
    /// the defalt value of the Stats type.
    pub fn stats(self) -> Stats {
        self.refresh_stats_cache();
        world::with(|w|
            if let Some(&Some(composite_stats)) = w.stats_caches().get(self) {
                composite_stats
            } else {
                self.base_stats()
            }
        )
    }

    /// Return the base stats, if any, of the entity. Does not try to generate
    /// or fetch composite stats.
    fn base_stats(self) -> Stats {
        world::with(|w|
            if let Some(s) = w.stats().get(self) {
                *s
            } else {
                Default::default()
            }
        )
    }

    /// Generate cached stats from base stats if they don't exist.
    /// This must be called by any method that accesses the stats_caches
    /// component.
    fn refresh_stats_cache(self) {
        // If stats cache doesn't exist, do nothing.
        world::with(|w| if w.stats_caches().get(self).is_none() { return; });

        // If cache is good, do nothing.
        world::with(|w| if let Some(&Some(_)) = w.stats_caches().get(self) { return; });

        let mut stats = self.base_stats();
        for &slot in [
            Slot::Body,
            Slot::Feet,
            Slot::Head,
            Slot::Melee,
            Slot::TrinketF,
            Slot::TrinketG,
            Slot::TrinketH,
            Slot::TrinketI].iter() {
            if let Some(item) = self.equipped(slot) {
                stats = stats + item.stats();
            }
        }

        world::with_mut(|w| w.stats_caches_mut().insert(self, Some(stats)));
    }

    /// Mark cached stats dirty after changing base stats.
    fn dirty_stats_cache(self) {
        world::with_mut(|w|
            w.stats_caches_mut().insert(self, None)
        );
    }

// Item methods ////////////////////////////////////////////////////////

    pub fn is_item(self) -> bool { world::with(|w| w.items().get(self).is_some()) }

    /// Is this an item that has an instant effect when stepped on.
    pub fn is_instant_item(self) -> bool {
        world::with(|w|
            if let Some(item) = w.items().get(self) {
                item.item_type == ItemType::Instant
            } else {
                false
            }
        )
    }

    pub fn can_be_picked_up(self) -> bool {
        world::with(|w|
            if let Some(item) = w.items().get(self) {
                item.item_type != ItemType::Instant
            } else {
                false
            }
        )
    }

    /// Preferred equipment slot for equippable items.
    pub fn equip_slots(self) -> Vec<Slot> {
        world::with(|w|
            if let Some(item) = w.items().get(self) {
                match item.item_type {
                    ItemType::MeleeWeapon => vec![Slot::Melee],
                    ItemType::RangedWeapon => vec![Slot::Ranged],
                    ItemType::Helmet => vec![Slot::Head],
                    ItemType::Armor => vec![Slot::Body],
                    ItemType::Boots => vec![Slot::Feet],
                    ItemType::Trinket => vec![
                        Slot::TrinketF, Slot::TrinketG, Slot::TrinketH, Slot::TrinketI],
                    ItemType::Spell => vec![
                        Slot::Spell1, Slot::Spell2, Slot::Spell3, Slot::Spell4,
                        Slot::Spell5, Slot::Spell6, Slot::Spell7, Slot::Spell8,
                    ],
                    _ => vec![]
                }
            } else {
                vec![]
            }
        )

    }

// AI methods /////////////////////////////////////////////////////////

    /// Top-level method called each frame to update the entity.
    pub fn update(self) {
        if self.is_mob() && !self.is_player() && self.ticks_this_frame() {
            self.mob_ai();
        }
    }

    fn brain_state(self) -> Option<BrainState> {
        world::with(|w| w.brains().get(self).map(|b| b.state))
    }

    fn set_brain_state(self, brain_state: BrainState) {
        world::with_mut(|w| w.brains_mut().get(self).expect("no brains").state = brain_state );
    }

    fn alignment(self) -> Option<Alignment> {
        world::with(|w| w.brains().get(self).map(|b| b.alignment))
    }

    fn wake_up(self) {
        if self.brain_state() == Some(BrainState::Asleep) {
            self.set_brain_state(BrainState::Hunting);
        }
    }

    /// AI routine for autonomous mobs.
    fn mob_ai(self) {
        assert!(self.is_mob());
        assert!(!self.is_player());
        assert!(self.ticks_this_frame());

        if self.brain_state() == Some(BrainState::Asleep) {
            if let Some(p) = action::player() {
                // TODO: Line-of-sight, stealth concerns, other enemies than
                // player etc.
                if let Some(d) = p.distance_from(self) {
                    if d < 6 {
                        self.wake_up();
                    }
                }
            }

            return;
        }

        if let Some(p) = action::player() {
            let loc = self.location().expect("no location");

            let vec_to_enemy = loc.v2_at(p.location().expect("no location"));
            if let Some(v) = vec_to_enemy {
                if v.hex_dist() == 1 {
                    // Melee range, hit.
                    self.melee(Dir6::from_v2(v));
                } else {
                    // Walk towards.
                    let pathing_depth = 16;
                    let pathing = Dijkstra::new(
                        vec![p.location().expect("no location")], |&loc| !loc.blocks_walk(),
                        pathing_depth);

                    let steps = pathing.sorted_neighbors(&loc);
                    if steps.len() > 0 {
                        self.step(loc.dir6_towards(steps[0]).expect("No loc pair orientation"));
                    } else {
                        self.step(rng::with(|ref mut rng| rng.gen::<Dir6>()));
                        // TODO: Fall asleep if things get boring.
                    }
                }
            }
        }
    }

    /// Return whether this thing wants to fight the other thing.
    fn is_hostile_to(self, other: Entity) -> bool {
        match (self.alignment(), other.alignment()) {
            (Some(Alignment::Chaotic), Some(_)) => true,
            (Some(_), Some(Alignment::Chaotic)) => true,
            (Some(Alignment::Evil), Some(Alignment::Good)) => true,
            (Some(Alignment::Good), Some(Alignment::Evil)) => true,
            _ => false,
        }
    }

    /// Return whether the mob has hostiles in its immediate vicinity.
    pub fn is_threatened(self) -> bool {
        // XXX: Expensive.
        let range = 6;
        let loc = self.location().expect("no location");
        let seen: Vec<Location> = Fov::new(
            |pt| (loc + pt).blocks_sight(), range)
            .map(|pt| loc + pt)
            .collect();
        for loc in seen.iter() {
            if let Some(m) = loc.mob_at() {
                if m.is_hostile_to(self) {
                    return true;
                }
            }
        }
        false
    }

// Callbacks ///////////////////////////////////////////////////////////

    /// Called after the entity is moved to a new location.
    pub fn on_move_to(self, loc: Location) {
        self.do_fov();

        for &e in loc.entities().iter() {
            if e != self {
                e.on_step_on(self);
            }
        }

        if self.is_player() {
            if loc.terrain().is_exit() {
                action::next_level();
            }
        }
    }

    /// When another entity steps on this one. Useful for traps and
    /// instaeffect items.
    pub fn on_step_on(self, collider: Entity) {
        if self.is_instant_item() {
            let ability = world::with(|w| w.items().get(self).expect("no item").ability.clone());
            ability.apply(Some(self), Place::In(collider, None));
        }
    }

// FOV and map memory //////////////////////////////////////////////////

    fn has_map_memory(self) -> bool {
        world::with(|w| w.map_memories().get(self).is_some())
    }

    fn do_fov(self) {
        let range = 12;
        if let Some(loc) = self.location() {
            if self.has_map_memory() {
                let seen: Vec<Location> = Fov::new(
                    |pt| (loc + pt).blocks_sight(), range)
                    .map(|pt| loc + pt)
                    .collect();
                world::with_mut(|w| {
                    if let Some(ref mut mm) = w.map_memories_mut().get(self) {
                        mm.seen.clear();
                        mm.seen.extend(seen.clone().into_iter());
                        mm.remembered.extend(seen.iter().map(|&x| x));
                    } else {
                        panic!("Couldn't bind map memory");
                    }
                });
            }
        }
    }

    pub fn forget_map(self) {
        if self.has_map_memory() {
            world::with_mut(|w| {
                if let Some(ref mut mm) = w.map_memories_mut().get(self) {
                    mm.seen.clear();
                    mm.remembered.clear();
                } else {
                    panic!("Couldn't bind map memory");
                }
            });
        }
    }
}
