use calx::dijkstra::Dijkstra;
use world;
use location::{Location};
use dir6::Dir6;
use flags;
use mob;
use mob::status;
use mob::intrinsic;
use super::MobKind;
use spatial;
use action;

/// Game object handle.
#[deriving(PartialEq, Eq, Clone, Hash, Show, Decodable, Encodable)]
pub struct Entity(pub uint);

impl Entity {
    /// Place the entity in a location in the game world.
    pub fn place(self, loc: Location) {
        world::get().borrow_mut().spatial.insert_at(self, loc);
    }

    /// Remove the entity from all the game world systems. THE ENTITY VALUE
    /// WILL BE INVALID FROM HERE ON AND USING IT WILL LEAD TO BUGS. THE
    /// CALLER IS RESPONSIBLE FOR ENSUING THAT AN ENTITY WILL NOT BE
    /// USED FROM ANYWHERE AFTER THE DELETE OPERATION.
    pub fn delete(self) {
        // LABYRINTH OF COMPONENTS
        // This needs to call every toplevel component system.
        world::get().borrow_mut().comp.remove(self);
        world::get().borrow_mut().spatial.remove(self);
        world::get().borrow_mut().ecs.delete(self);
    }

    pub fn blocks_walk(self) -> bool { self.is_mob() }

    /// Return the kind of the entity.
    pub fn kind(self) -> ::EntityKind {
        // XXX: Will crash if an entity has no kind specified.
        *world::get().borrow().comp.kind.get(self).unwrap()
    }

// Spatial methods /////////////////////////////////////////////////////

    pub fn can_enter(self, loc: Location) -> bool {
        if self.is_mob() && loc.has_mobs() { return false; }
        if loc.blocks_walk() { return false; }
        true
    }

    /// Return whether the entity can move in a direction.
    pub fn can_step(self, dir: Dir6) -> bool {
        let place = world::get().borrow().spatial.get(self);
        if let Some(spatial::At(loc)) = place {
            let new_loc = loc + dir.to_v2();
            return self.can_enter(new_loc);
        }
        return false;
    }

    /// Try to move the entity in direction.
    pub fn step(self, dir: Dir6) {
        let place = world::get().borrow().spatial.get(self);
        if let Some(spatial::At(loc)) = place {
            let new_loc = loc + dir.to_v2();
            if self.can_enter(new_loc) {
                world::get().borrow_mut().spatial.insert_at(self, new_loc);
            }
        }
    }

    pub fn location(self) -> Option<Location> {
        match world::get().borrow().spatial.get(self) {
            Some(spatial::At(loc)) => Some(loc),
            Some(spatial::In(e)) => e.location(),
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

// Mob methods /////////////////////////////////////////////////////////

    pub fn is_mob(self) -> bool {
        if let Some(&MobKind(_)) = world::get().borrow().comp.kind.get(self) {
            return true;
        }
        return false;
    }

    /// Return whether this mob is the player avatar.
    pub fn is_player(self) -> bool {
        if let Some(&MobKind(mob::Player)) = world::get().borrow().comp.kind.get(self) {
            return true;
        }
        return false;
    }

    pub fn has_status(self, status: status::Status) -> bool {
        if let Some(&mob) = world::get().borrow().comp.mob.get(self) {
            return mob.has_status(status);
        }
        return false;
    }

    pub fn add_status(self, status: status::Status) {
        if !self.is_mob() { return; }
        world::get().borrow_mut().comp.mob.get_mut(self).unwrap().add_status(status);
        assert!(self.has_status(status));
    }

    pub fn remove_status(self, status: status::Status) {
        if !self.is_mob() { return; }
        world::get().borrow_mut().comp.mob.get_mut(self).unwrap().remove_status(status);
        assert!(!self.has_status(status));
    }

    pub fn has_intrinsic(self, intrinsic: mob::intrinsic::Intrinsic) -> bool {
        if let Some(&MobKind(mt)) = world::get().borrow().comp.kind.get(self) {
            return mob::SPECS[mt as uint].intrinsics & intrinsic as int != 0;
        }
        return false;
    }

    /// Return whether this entity is an awake mob.
    pub fn is_active(self) -> bool {
        self.is_mob() && !self.has_status(status::Asleep)
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
            1 => return self.has_intrinsic(intrinsic::Fast),
            2 => return true,
            3 => return self.has_status(status::Quick),
            4 => return !self.has_intrinsic(intrinsic::Slow)
                        && !self.has_status(status::Slow),
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

        if self.has_status(status::Asleep) {
            if let Some(p) = action::player() {
                // TODO: Line-of-sight, stealth concerns, other enemies than
                // player etc.
                if let Some(d) = p.distance_from(self) {
                    if d < 6 {
                        self.remove_status(status::Asleep);
                    }
                }
            }

            return;
        }

        if let Some(p) = action::player() {
            let pathing = Dijkstra::new(vec![p.location().unwrap()], |&loc| !loc.blocks_walk(), 64);
            let loc = self.location().unwrap();

            let steps = pathing.sorted_neighbors(&loc);
            self.step(loc.dir6_towards(steps[0]).unwrap());
        }
    }
}
