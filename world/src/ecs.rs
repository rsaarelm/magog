use world;
use entity::{Entity};

/// Entity component system.
#[deriving(Decodable, Encodable)]
pub struct Ecs {
    next_idx: uint,
    reusable_idxs: Vec<uint>,
    // Could use Bitv for active, but I can't bother to write the serializer...
    active: Vec<bool>,
}

impl Ecs {
    pub fn new() -> Ecs {
        Ecs {
            next_idx: 0,
            reusable_idxs: vec![],
            active: vec![],
        }
    }

    pub fn new_entity(&mut self) -> Entity {
        // Get the entity idx, reuse old ones to keep the indexing compact.
        let idx = match self.reusable_idxs.pop() {
            None => {
                let ret = self.next_idx;
                self.next_idx += 1;
                ret
            }
            Some(idx) => idx
        };

        if self.active.len() <= idx {
            let diff = idx - self.active.len() + 1;
            self.active.grow(diff, false);
        }
        assert!(!self.active[idx]);
        self.active[idx] = true;

        Entity(idx)
    }

    /// Delete an entity from the entity component system.
    ///
    /// XXX: The user is currently responsible for never using an entity
    /// handle again after delete_entity has been called on it. Using an
    /// entity handle after deletion may return another entity's contents.
    pub fn delete(&mut self, Entity(idx): Entity) {
        assert!(self.active[idx]);

        self.reusable_idxs.push(idx);
        self.active[idx] = false;
    }

    /// Return an iterator for the entities. The iterator will not be
    /// invalidated if entities are added or removed during iteration. The
    /// iterator also won't maintain a lock on the world singleton outside
    /// calling next.
    ///
    /// XXX: It is currently unspecified whether entities added during
    /// iteration will show up in the iteration or not.
    pub fn iter(&self) -> EntityIter {
        EntityIter(0)
    }
}

pub struct EntityIter(uint);

impl Iterator<Entity> for EntityIter {
    fn next(&mut self) -> Option<Entity> {
        let w = world::get();
        let ecs = &w.borrow().ecs;

        let &EntityIter(ref mut idx) = self;
        loop {
            if *idx >= ecs.active.len() { return None; }
            let ret = Entity(*idx);
            *idx += 1;
            if !ecs.active[*idx - 1] { continue; }
            return Some(ret);
        }
    }
}

/// Generic component type that holds some simple data elements associated
/// with entities.
#[deriving(Encodable, Decodable)]
pub struct Component<T> {
    data: Vec<Option<T>>,
}

impl<T> Component<T> {
    pub fn new() -> Component<T> {
        Component {
            data: vec![],
        }
    }

    /// Remove an entity's element.
    pub fn remove(&mut self, Entity(idx): Entity) {
        if idx < self.data.len() {
            self.data[idx] = None;
        }
    }

    /// Insert an element for an entity.
    pub fn insert(&mut self, Entity(idx): Entity, c: T) {
        while self.data.len() <= idx { self.data.push(None); }
        self.data[idx] = Some(c);
    }

    /// Get the element for an entity if it exists.
    pub fn get<'a>(&'a self, Entity(idx): Entity) -> Option<&'a T> {
        if idx >= self.data.len() { return None; }
        self.data[idx].as_ref()
    }

    /// Get a mutable reference to the element for an entity if it exists.
    pub fn get_mut<'a>(&'a mut self, Entity(idx): Entity) -> Option<&'a mut T> {
        if idx >= self.data.len() { return None; }
        self.data[idx].as_mut()
    }
}
