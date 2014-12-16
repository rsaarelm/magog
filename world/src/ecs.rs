use std::rc::Rc;
use std::cell::RefCell;
use std::collections::VecMap;
use world;
use entity::{Entity};

/// Entity component system.
#[deriving(Decodable, Encodable)]
pub struct Ecs {
    next_idx: uint,
    reusable_idxs: Vec<uint>,
    // Could use Bitv for active, but I can't bother to write the serializer...
    active: Vec<bool>,
    parent: VecMap<uint>,
}

impl Ecs {
    pub fn new() -> Ecs {
        Ecs {
            next_idx: 0,
            reusable_idxs: vec![],
            active: vec![],
            parent: VecMap::new(),
        }
    }

    pub fn new_entity(&mut self, parent: Option<Entity>) -> Entity {
        // Get the entity idx, reuse old ones to keep the indexing compact.
        let idx = match self.reusable_idxs.pop() {
            None => {
                let ret = self.next_idx;
                self.next_idx += 1;
                ret
            }
            Some(idx) => idx
        };

        if let Some(Entity(p_idx)) = parent {
            assert!(self.active[p_idx]);
            self.parent.insert(idx, p_idx);
        }

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

        self.parent.remove(&idx);
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
        world::with(|w| {
            let &EntityIter(ref mut idx) = self;
            loop {
                if *idx >= w.ecs.borrow().active.len() { return None; }
                let ret = Entity(*idx);
                *idx += 1;
                if !w.ecs.borrow().active[*idx - 1] { continue; }
                return Some(ret);
            }
        })
    }
}

/// Generic component type that holds some simple data elements associated
/// with entities.
#[deriving(Encodable, Decodable)]
pub struct Component<T> {
    data: VecMap<T>,
    ecs: Rc<RefCell<Ecs>>,
}

impl<T> Component<T> {
    pub fn new(ecs: Rc<RefCell<Ecs>>) -> Component<T> {
        Component {
            data: VecMap::new(),
            ecs: ecs,
        }
    }

    /// Remove an entity's element.
    pub fn remove(&mut self, Entity(idx): Entity) { self.data.remove(&idx); }

    /// Insert an element for an entity.
    pub fn insert(&mut self, Entity(idx): Entity, c: T) { self.data.insert(idx, c); }

    /// Get the element for an entity if it exists.
    pub fn get<'a>(&'a self, Entity(idx): Entity) -> Option<&'a T> {
        match self.data.get(&idx) {
            None => {
                if let Some(&parent_idx) = self.ecs.borrow().parent.get(&idx) {
                    self.get(Entity(parent_idx))
                } else {
                    None
                }
            }
            ret => ret
        }
    }

    /// Get a mutable reference to the element for an entity if it exists.
    pub fn get_mut<'a>(&'a mut self, Entity(idx): Entity) -> Option<&'a mut T> {
        if !self.data.contains_key(&idx) {
            let parent = self.ecs.borrow().parent.get(&idx).map(|&x| x);
            match parent {
                Some(parent_idx) => { return self.get_mut(Entity(parent_idx)); }
                None => { return None; }
            }
        }
        self.data.get_mut(&idx)
    }
}
