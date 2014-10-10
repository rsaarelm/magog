use serialize::{Decodable, Decoder, Encodable, Encoder};
use std::intrinsics::TypeId;
use std::collections::hashmap::HashMap;
use std::collections::bitv::Bitv;
use std::any::{Any, AnyRefExt, AnyMutRefExt};
use world;

#[deriving(PartialEq, Clone, Hash, Show)]
pub struct Entity(uint);

/// Entity component system
pub struct Ecs {
    // XXX: This is much less efficient than it could be. A serious
    // implementation would use unboxed vecs for the components and would
    // provide lookup methods faster than a HashMap find to access the
    // components
    components: HashMap<TypeId, Vec<Option<Box<Any + 'static>>>>,

    next_idx: uint,
    reusable_idxs: Vec<uint>,
    active: Bitv,
}

impl Ecs {
    pub fn new() -> Ecs {
        Ecs {
            components: HashMap::new(),
            next_idx: 0,
            reusable_idxs: vec![],
            active: Bitv::new(),
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
        assert!(!self.active.get(idx));
        self.active.set(idx, true);

        Entity(idx)
    }

    /// Delete an entity from the entity component system.
    ///
    /// XXX: The user is currently responsible for never using an entity handle again after
    /// delete_entity has been called on it. Using an entity handle after deletion may return
    /// another entity's contents.
    pub fn delete_entity(&mut self, Entity(idx): Entity) {
        assert!(self.active.get(idx));

        for (_, c) in self.components.iter_mut() {
            if c.len() > idx {
                c.as_mut_slice()[idx] = None;
            }
        }

        self.reusable_idxs.push(idx);
        self.active.set(idx, false);
    }

    /// Return an iterator for the entities. The iterator will not be
    /// invalidated if entities are added or removed during iteration.
    ///
    /// XXX: It is currently unspecified whether entities added during
    /// iteration will show up in the iteration or not.
    pub fn iter(&self) -> EntityIter {
        EntityIter(0)
    }
}

impl<E, D:Decoder<E>> Decodable<D, E> for Ecs {
    fn decode(d: &mut D) -> Result<Ecs, E> {
        unimplemented!();
    }
}

impl<E, S:Encoder<E>> Encodable<S, E> for Ecs {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        unimplemented!();
    }
}

struct EntityIter(uint);

impl Iterator<Entity> for EntityIter {
    fn next(&mut self) -> Option<Entity> {
        let w = world::get();
        let ecs = &w.borrow().ecs;

        let &EntityIter(ref mut idx) = self;
        loop {
            if *idx >= ecs.active.len() { return None; }
            let ret = Entity(*idx);
            *idx += 1;
            if !ecs.active.get(*idx - 1) { continue; }
            return Some(ret);
        }
    }
}
