use world;
use location::{Location};

/// Game object handle.
#[deriving(PartialEq, Eq, Clone, Hash, Show, Decodable, Encodable)]
pub struct Entity(pub uint);

impl Entity {
    /// Place the entity in a location in the game world.
    pub fn place(self, loc: Location) {
        world::get().borrow_mut().spatial.insert(self, loc);
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

    /// Return the kind of the entity.
    pub fn kind(self) -> ::EntityKind {
        // XXX: Will crash if an entity has no kind specified.
        *world::get().borrow().comp.kind.get(self).unwrap()
    }

    /// Return whether the entity is an awake non-player mob and should be
    /// animated with a bob.
    pub fn is_bobbing(self) -> bool {
        // TODO
        false
    }
}
