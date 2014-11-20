use std::rand::Rng;
use calx::dijkstra::Dijkstra;
use world;
use location::{Location};
use dir6::Dir6;
use flags;
use mob;
use mob::{Status, Intrinsic, MobType};
use geom::HexGeom;
use spatial::Place;
use action;
use fov::Fov;
use rng;
use msg;

/// Game object handle.
#[deriving(PartialEq, Eq, Clone, Hash, Show, Decodable, Encodable)]
pub struct Entity(pub uint);

impl Entity {
    /// Place the entity in a location in the game world.
    pub fn place(self, loc: Location) {
        world::with_mut(|w| w.spatial.insert_at(self, loc));
        self.on_move_to(loc);
    }

    /// Remove the entity from all the game world systems. THE ENTITY VALUE
    /// WILL BE INVALID FROM HERE ON AND USING IT WILL LEAD TO BUGS. THE
    /// CALLER IS RESPONSIBLE FOR ENSUING THAT AN ENTITY WILL NOT BE
    /// USED FROM ANYWHERE AFTER THE DELETE OPERATION.
    pub fn delete(self) {
        // LABYRINTH OF COMPONENTS
        // This needs to call every toplevel component system.
        world::with_mut(|w| w.comp.remove(self));
        world::with_mut(|w| w.spatial.remove(self));
        world::with_mut(|w| w.ecs.delete(self));
    }

    pub fn blocks_walk(self) -> bool { self.is_mob() }

    /// Return the kind of the entity.
    pub fn kind(self) -> ::EntityKind {
        // XXX: Will crash if an entity has no kind specified.
        world::with(|w| *w.comp.kind.get(self).unwrap())
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
            Some(Place::In(e)) => e.location(),
            _ => None
        }
    }

    pub fn distance_from(self, other: Entity) -> Option<int> {
        if let (Some(loc1), Some(loc2)) = (self.location(), other.location()) {
            loc1.distance_from(loc2)
        } else {
            None
        }
    }

// Damage and lifetime /////////////////////////////////////////////////

    pub fn damage(self, amount: int) {
        if amount == 0 { return; }

        let (_amount, kill) = world::with_mut(|w| {
            if let Some(&mut mob) = w.comp.mob.get_mut(self) {
                mob.hp -= amount;
                if mob.hp <= 0 {
                    // Remember this when we're out of the borrow.
                    return (amount, true);
                }
            } else {
                panic!("Damaging a non-mob");
            };
            (amount, false)
        });

        msg::push_msg(::Msg::Damage(self));
        if kill {
            self.kill();
        }
    }

    /// Do any game logic stuff related to this entity dying violently before
    /// deleting it.
    pub fn kill(self) {
        msg::push_msg(::Msg::Gib(self.location().unwrap()));
        self.delete();
    }

// Mob methods /////////////////////////////////////////////////////////

    pub fn is_mob(self) -> bool {
        world::with(|w| {
            if let Some(&::EntityKind::Mob(_)) = w.comp.kind.get(self) {
                return true;
            }
            return false;
        })
    }

    /// Return whether this mob is the player avatar.
    pub fn is_player(self) -> bool {
        world::with(|w| {
            if let Some(&::EntityKind::Mob(MobType::Player)) = w.comp.kind.get(self) {
                return true;
            }
            return false;
        })
    }

    pub fn has_status(self, status: Status) -> bool {
        world::with(|w| {
            if let Some(&mob) = w.comp.mob.get(self) {
                return mob.has_status(status);
            }
            return false;
        })
    }

    pub fn add_status(self, status: Status) {
        if !self.is_mob() { return; }
        world::with_mut(|w| w.comp.mob.get_mut(self).unwrap().add_status(status));
        assert!(self.has_status(status));
    }

    pub fn remove_status(self, status: Status) {
        if !self.is_mob() { return; }
        world::with_mut(|w| w.comp.mob.get_mut(self).unwrap().remove_status(status));
        assert!(!self.has_status(status));
    }

    pub fn has_intrinsic(self, intrinsic: Intrinsic) -> bool {
        world::with(|w| {
            if let Some(&::EntityKind::Mob(mt)) = w.comp.kind.get(self) {
                return mob::MOB_SPECS[mt as uint].intrinsics & intrinsic as int != 0;
            }
            return false;
        })
    }

    /// Return whether this entity is an awake mob.
    pub fn is_active(self) -> bool {
        self.is_mob() && !self.has_status(Status::Asleep)
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
            3 => return self.has_status(Status::Quick),
            4 => return !self.has_intrinsic(Intrinsic::Slow)
                        && !self.has_status(Status::Slow),
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
        let loc = self.location().unwrap() + dir.to_v2();
        if let Some(e) = loc.mob_at() {
            // Attack power
            let p = world::with(|w| w.comp.mob.get(self).unwrap().power);
            if p == 0 {
                // No fight capacity.
                return;
            }

            // Every five points of power is one certain hit.
            let full = p / 5;
            // The fractional points are one probabilistic hit.
            let partial = (p % 5) as f64 / 5.0;

            let damage = full + if rng::p(partial) { 1 } else { 0 };
            e.damage(damage)
        }
    }

// AI methods /////////////////////////////////////////////////////////

    /// Top-level method called each frame to update the entity.
    pub fn update(self) {
        if self.is_mob() && !self.is_player() && self.ticks_this_frame() {
            self.mob_ai();
        }
    }

    /// AI routine for autonomous mobs.
    fn mob_ai(self) {
        assert!(self.is_mob());
        assert!(!self.is_player());
        assert!(self.ticks_this_frame());

        if self.has_status(Status::Asleep) {
            if let Some(p) = action::player() {
                // TODO: Line-of-sight, stealth concerns, other enemies than
                // player etc.
                if let Some(d) = p.distance_from(self) {
                    if d < 6 {
                        self.remove_status(Status::Asleep);
                    }
                }
            }

            return;
        }

        if let Some(p) = action::player() {
            let loc = self.location().unwrap();

            let vec_to_enemy = loc.v2_at(p.location().unwrap());
            if let Some(v) = vec_to_enemy {
                if v.hex_dist() == 1 {
                    // Melee range, hit.
                    self.melee(Dir6::from_v2(v));
                } else {
                    // Walk towards.
                    let pathing_depth = 16;
                    let pathing = Dijkstra::new(
                        vec![p.location().unwrap()], |&loc| !loc.blocks_walk(),
                        pathing_depth);

                    let steps = pathing.sorted_neighbors(&loc);
                    if steps.len() > 0 {
                        self.step(loc.dir6_towards(steps[0]).unwrap());
                    } else {
                        self.step(rng::with(|ref mut rng| rng.gen::<Dir6>()));
                        // TODO: Fall asleep if things get boring.
                    }
                }
            }
        }
    }

// Callbacks ///////////////////////////////////////////////////////////

    /// Called after the entity is moved to a new location.
    pub fn on_move_to(self, _loc: Location) {
        self.do_fov();
    }

// Misc ////////////////////////////////////////////////////////////////

    fn has_map_memory(self) -> bool {
        world::with(|w| w.comp.map_memory.get(self).is_some())
    }

    fn do_fov(self) {
        let range = 12u;
        if let Some(loc) = self.location() {
            if self.has_map_memory() {
                let seen: Vec<Location> = Fov::new(
                    |pt| (loc + pt).blocks_sight(), range)
                    .map(|pt| loc + pt)
                    .collect();
                world::with_mut(|w| {
                    if let Some(ref mut mm) = w.comp.map_memory.get_mut(self) {
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
}
