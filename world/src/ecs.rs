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

    /// Return the optional parent entity of an entity.
    pub fn parent(&self, Entity(idx): Entity) -> Option<Entity> {
        self.parent.get(&idx).map(|&idx| Entity(idx))
    }
}

pub struct EntityIter(uint);

impl Iterator<Entity> for EntityIter {
    fn next(&mut self) -> Option<Entity> {
        world::with(|w| {
            let &EntityIter(ref mut idx) = self;
            loop {
                if *idx >= w.ecs.active.len() { return None; }
                let ret = Entity(*idx);
                *idx += 1;
                if !w.ecs.active[*idx - 1] { continue; }
                return Some(ret);
            }
        })
    }
}
