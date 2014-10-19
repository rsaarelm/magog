use world;
use location::{Location};
use dir6::Dir6;
use flags;
use mob;
use mob::status;
use mob::intrinsic;
use super::MobKind;
use spatial;

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

    pub fn is_mob(self) -> bool {
        if let Some(&MobKind(_)) = world::get().borrow().comp.kind.get(self) {
            return true;
        }
        return false;
    }

    /// Return the kind of the entity.
    pub fn kind(self) -> ::EntityKind {
        // XXX: Will crash if an entity has no kind specified.
        *world::get().borrow().comp.kind.get(self).unwrap()
    }

    /// Return whether the entity is an awake non-player mob and should be
    /// animated with a bob.
    pub fn is_bobbing(self) -> bool {
        self.is_active() && !self.is_player()
    }

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

    /// Return whether this mob is the player avatar.
    pub fn is_player(self) -> bool {
        if let Some(&MobKind(mob::Player)) = world::get().borrow().comp.kind.get(self) {
            return true;
        }
        return false;
    }

    pub fn has_status(self, status: mob::status::Status) -> bool {
        if let Some(&mob) = world::get().borrow().comp.mob.get(self) {
            return mob.has_status(status);
        }
        return false;
    }

    pub fn has_intrinsic(self, intrinsic: mob::intrinsic::Intrinsic) -> bool {
        if let Some(&MobKind(mt)) = world::get().borrow().comp.kind.get(self) {
            return mob::SPECS[mt as uint].intrinsics & intrinsic as int != 0;
        }
        return false;
    }

    /// Return whether this entity is an awake mob.
    pub fn is_active(self) -> bool {
        self.is_mob() && !self.has_status(mob::status::Asleep)
    }

    /// Return whether the entity is a mob that will act this frame.
    pub fn acts_this_frame(self) -> bool {
        if !self.is_active() { return false; }
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
            _ => fail!("Invalid action phase"),
        }
    }

    pub fn location(self) -> Option<Location> {
        match world::get().borrow().spatial.get(self) {
            Some(spatial::At(loc)) => Some(loc),
            Some(spatial::In(e)) => e.location(),
            _ => None
        }
    }
}
