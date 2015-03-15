use std::default::Default;
use util::Dijkstra;
use util::Rgb;
use util::color;
use world;
use location::{Location};
use dir6::Dir6;
use flags;
use components::{BrainState, Alignment, Brain};
use geom::HexGeom;
use spatial::Place;
use action;
use fov::Fov;
use rng;
use msg;
use item::{ItemType, Slot};
use stats::{Stats, Intrinsic};
use ecs::{ComponentAccess};
use terrain::TerrainType;

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
        world::with_mut(|w|
            if w.flags.player == Some(self) { w.flags.player = None; });
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

        if ret.is_terran() {
            world::with_mut(|w| w.flags.terrans_left += 1);
        }

        ret
    }

    pub fn is_prototype(self) -> bool {
        world::with(|w| w.prototypes().get_local(self).is_some())
    }

    pub fn parent(self) -> Option<Entity> {
        world::with(|w| w.ecs.parent(self))
    }

    pub fn reparent(self, new_parent: Entity) {
        assert!(new_parent.is_prototype());
        world::with_mut(|w| w.ecs.reparent(self, new_parent));
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
            return self.can_enter(new_loc) ||
                (self.is_player() && new_loc.terrain() == TerrainType::Door);
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
            } else if new_loc.terrain() == TerrainType::Door && self.is_player() {
                // Player can force doors even in unsuitable form.
                let force_difficulty = 5 - self.stats().power / 2;
                if force_difficulty <= 1 || rng::one_chance_in(force_difficulty as u32) {
                    world::with_mut(|w| w.spatial.insert_at(self, new_loc));
                    self.on_move_to(new_loc);
                    msgln!("Door forced.");
                } else {
                    msgln!("Morph has trouble with doors.");
                }
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

        if self.is_player() && !self.is_exposed_phage() {
            // Phage is just re-exposed when host dies.
            self.exit_host();
            return;
        }

        if self.is_player() {
            caption!("Phage lost");
            action::delete_save();
        } else if self.has_intrinsic(Intrinsic::Robotic) {
            msgln!("{} destroyed.", capitalize(&self.name()));
        } else {
            msgln!("{} dies.", capitalize(&self.name()));
        }

        if self.is_terran() {
            let terrans_left = world::with_mut(|w| {
                w.flags.terrans_left -= 1;
                w.flags.terrans_left
            });

            if terrans_left == 0 {
                caption!("Zero terran DNA signatures detected. Phage has secured the zone.");
            }
        }

        msg::push(::Msg::Gib(loc));

        // Turn into corpse.
        world::with_mut(|w| {
            w.brains_mut().hide(self);
            // Corpses are always icon + 1.
            w.descs_mut().get(self).expect("no desc").icon += 1;
        });

        // Try to have one corpse per cell, spill out if dying on top of
        // another corpse. (If there's no room left around, the corpses will
        // just stack.)
        if let Some(loc) = self.location().unwrap().spill(
            |loc| self.can_enter(loc) &&
            loc.entities().iter().find(|&x| x != &self && x.is_corpse()).is_none()) {
            self.place(loc);
        }

        self.set_intrinsic(Intrinsic::Dead);
        //self.delete();
    }

// Mob methods /////////////////////////////////////////////////////////

    pub fn is_corpse(self) -> bool { self.has_intrinsic(Intrinsic::Dead) }

    pub fn is_mob(self) -> bool {
        world::with(|w| w.brains().get(self).is_some()) && self.location().is_some()
    }

    /// Return whether this mob is the player avatar.
    pub fn is_player(self) -> bool {
        self.brain_state() == Some(BrainState::PlayerControl) && self.location().is_some()
    }

    /// Return whether this entity is an awake mob.
    pub fn is_active(self) -> bool {
        if self.has_intrinsic(Intrinsic::Dead) {
            return false;
        }

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
        if !self.is_mob() { return false; }

        let tick = flags::get_tick();
        // Go through a cycle of 5 phases to get 4 possible speeds.
        // System idea from Jeff Lait.
        let phase = tick % 5;
        match phase {
            0 => return true,
            1 => return self.has_intrinsic(Intrinsic::Fast),
            2 => return true,
            3 => return self.has_intrinsic(Intrinsic::Quick),
            4 => return !self.has_intrinsic(Intrinsic::Slow),
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

    pub fn shoot(self, dir: Dir6) {
        let stats = self.stats();

        if stats.ranged_range > 0 {
            action::shoot(self.location().unwrap(), dir, stats.ranged_range, stats.ranged_power);
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

    pub fn set_intrinsic(self, intrinsic: Intrinsic) {
        world::with_mut(|w|
            if let Some(x) = w.stats_mut().get(self) {
                x.intrinsics |= intrinsic as u32;
            }
        );
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
            Slot::Ranged,
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

        if self.is_player() {
            if !self.is_exposed_phage() {
                // Host rots slowly.
                if rng::one_chance_in(64) {
                    self.apply_damage(1);
                }
            } else {
                // Exposed phage regenerates
                if rng::one_chance_in(6) {
                    self.heal(1);
                }
            }
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
                    if d < 8 && rng::one_chance_in((d / 2) as u32 + 1) {
                        self.wake_up();
                    }
                }
            }

            return;
        }

        // Start hunting nearby enemy.
        if self.brain_state() == Some(BrainState::Roaming) {
            if let Some(p) = action::player() {
                if !p.is_corpse() {
                    if let Some(d) = p.distance_from(self) {
                        // TODO: Line-of-sight
                        if d < 6 {
                            self.set_brain_state(BrainState::Hunting);
                        }
                    }
                }
            }
        }

        if self.brain_state() == Some(BrainState::Roaming) {
            self.step(rng::gen());
            if rng::one_chance_in(32) { self.set_brain_state(BrainState::Asleep); }
            return;
        }

        if self.brain_state() == Some(BrainState::Hunting) {
            // TODO: Fight other mobs than player.
            if let Some(p) = action::player() {
                if p.is_corpse() {
                    self.set_brain_state(BrainState::Roaming);
                }

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
                            self.step(rng::gen());
                        }
                    }
                }
            }
        }
    }

    /// Return whether this thing wants to fight the other thing.
    pub fn is_hostile_to(self, other: Entity) -> bool {
        match (self.alignment(), other.alignment()) {
            (Some(x), Some(y)) if x != y => true,
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
            flags::set_camera(self.location().expect("No player location"));
        }
    }

    /// When another entity steps on this one. Useful for traps and
    /// instaeffect items.
    pub fn on_step_on(self, collider: Entity) {
        if self.is_instant_item() {
            let ability = world::with(|w| w.items().get(self).expect("no item").ability.clone());
            ability.apply(Some(self), Place::In(collider, None));
        }

        if collider.is_player() && self.is_corpse() && !self.has_intrinsic(Intrinsic::Robotic) {
            msgln!("Inhabiting {}.", self.name());
            collider.possess(self);
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

// Phage stuff /////////////////////////////////////////////////////////

    pub fn is_terran(self) -> bool { world::with(|w| w.colonists().get(self).is_some()) }

    /// Self is the exposed phage form, not possessing a host.
    pub fn is_exposed_phage(self) -> bool {
        // Hacky, just check the icon index.
        world::with(|w| w.descs().get(self).map_or(false, |d| d.icon == 40))
    }

    /// Make the phage possess the target host.
    pub fn possess(self, target: Entity) {
        // Hairy ECS trickery to do polymorph.
        self.reparent(target.parent().unwrap());

        world::with_mut(|w| {
            // Remove the local description for the previous form.
            w.descs_mut().clear(self);
            // Get the prototype description from new parent, with
            // copy-on-write. Modify it for phage look.
            {
                let desc = w.descs_mut().get(self).expect("No prototype desc");
                desc.name = "phage".to_string();
                desc.color = color::CYAN;
            }

            // Central nervous system bypass.
            w.brains_mut().insert(
                self,
                Brain {
                    state: BrainState::PlayerControl,
                    alignment: Alignment::Phage,
                });

            // Tissue regeneration.
            w.healths_mut().get(self).expect("no health").wounds = 0;

            // Adrenal overload.
            w.stats_mut().clear(self);
            w.stats_mut().get(self).expect("no stats").attack += 3;
        });

        self.dirty_stats_cache();

        if !self.is_exposed_phage() {
            // Discard previous host.
            msg::push(::Msg::Gib(self.location().unwrap()));
        }

        let loc = target.location().unwrap();
        target.delete();
        self.place(loc);
    }

    /// Exist a host body and revert to phage form.
    pub fn exit_host(self) {
        assert!(self.is_player() && !self.is_exposed_phage());

        self.reparent(action::find_prototype("phage").expect("No player prototype"));

        world::with_mut(|w| {
            // Remove custom desc.
            w.descs_mut().clear(self);
            // Remove custom stats.
            w.stats_mut().clear(self);
            // Go full health.
            w.healths_mut().get(self).expect("no health").wounds = 0;
        });
        self.dirty_stats_cache();

        // Gib fx from the host body.
        msg::push(::Msg::Gib(self.location().unwrap()));
        msgln!("Morph lost.");
    }
}

// TODO: Put in library
fn capitalize(string: &str) -> String {
    string.chars().enumerate()
        .map(|(i, c)| if i == 0 { c.to_uppercase().next().unwrap() } else { c })
        .collect::<String>()
}
