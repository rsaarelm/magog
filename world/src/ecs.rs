use std::collections::VecMap;
use world;
use entity::{Entity};
use components;
use component_ref::{ComponentRef, ComponentRefMut};
use stats;
use world::{WorldState};

/// Entity component system.
#[derive(RustcDecodable, RustcEncodable)]
pub struct Ecs {
    next_idx: usize,
    reusable_idxs: Vec<usize>,
    // Could use Bitv for active, but I can't bother to write the serializer...
    active: Vec<bool>,
    parent: VecMap<usize>,
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
            self.active.resize(idx + 1, false);
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

    /// Return the optional parent entity of an entity.
    pub fn parent(&self, Entity(idx): Entity) -> Option<Entity> {
        self.parent.get(&idx).map(|&idx| Entity(idx))
    }

    /// Change the parent of a live entity
    pub fn reparent(&mut self, Entity(idx): Entity, Entity(new_parent_idx): Entity) {
        self.parent.insert(idx, new_parent_idx);
    }
}

pub struct EntityIter(usize);

impl Iterator for EntityIter {
    type Item = Entity;
    fn next(&mut self) -> Option<Entity> {
        world::with(|w| {
            let &mut EntityIter(ref mut idx) = self;
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

////////////////////////////////////////////////////////////////////////

// The one big macro for defining the full set of available entity components
// in one place.
macro_rules! components {
    {
        // Declare the list of types which are included as components in the
        // game's entity component system. Also declare the non-mutable and
        // mutable accessor names for them. Example
        //
        // ```notrust
        //     [Mesh, meshes, meshes_mut],
        // ```
        $([$comp:ty, $access:ident, $access_mut:ident],)+
    } => {
        // The master container for all the components.
#[derive(RustcEncodable, RustcDecodable)]
        pub struct Comps {
            $($access: VecMap<$comp>,)+
        }

        /// Container for all regular entity components.
        impl Comps {
            pub fn new() -> Comps {
                Comps {
                    $($access: VecMap::new(),)+
                }
            }

            /// Remove the given entity from all the contained components.
            pub fn remove(&mut self, Entity(idx): Entity) {
                $(self.$access.remove(&idx);)+
            }
        }


        // Implement the Componet trait for the type, this provides an uniform
        // syntax for adding component values to entities used by the entity
        // factory.
        $(
            impl Component for $comp {
                // XXX: Figure out how to move self into the closure to
                // get rid of the .clone.
                fn add_to(self, e: Entity) { world::with_mut(|w| w.$access_mut().insert(e, self.clone())) }
            }
        )+


        // Implement the trait for accessing all the components that
        // WorldState will implement
        pub trait ComponentAccess<'a> {
            $(
            fn $access(&'a self) -> ComponentRef<'a, $comp>;
            fn $access_mut(&'a mut self) -> ComponentRefMut<'a, $comp>;
            )+
        }

        impl<'a> ComponentAccess<'a> for WorldState {
            $(
            fn $access(&'a self) -> ComponentRef<'a, $comp> {
                ComponentRef::new(&self.ecs, &self.comps.$access)
            }
            fn $access_mut(&'a mut self) -> ComponentRefMut<'a, $comp> {
                ComponentRefMut::new(&mut self.ecs, &mut self.comps.$access)
            }
            )+
        }
    }
}

pub trait Component {
    /// Create an uniform syntax for attaching components to entities to allow
    /// a fluent API for constructing prototypes.
    fn add_to(self, e: Entity);
}

// Component loadout for the game.
components! {
    [components::Desc, descs, descs_mut],
    [components::MapMemory, map_memories, map_memories_mut],
    [stats::Stats, stats, stats_mut],
    [components::Spawn, spawns, spawns_mut],
    [components::Health, healths, healths_mut],
    [components::Brain, brains, brains_mut],
    [components::Item, items, items_mut],
    [components::StatsCache, stats_caches, stats_caches_mut],
}
