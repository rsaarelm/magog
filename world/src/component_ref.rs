use std::collections::VecMap;
use ecs::Ecs;
use entity::Entity;

/// Immutable component access.
pub struct ComponentRef<'a, C: 'static> {
    ecs: &'a Ecs,
    data: &'a VecMap<C>,
}

impl<'a, C> ComponentRef<'a, C> {
    pub fn new(ecs: &'a Ecs, data: &'a VecMap<C>) -> ComponentRef<'a, C> {
        ComponentRef {
            ecs: ecs,
            data: data,
        }
    }

    /// Fetch a component from given entity or its parent.
    pub fn get(self, Entity(idx): Entity) -> Option<&'a C> {
        match find_parent(|Entity(idx)| self.data.contains_key(&idx), self.ecs, Entity(idx)) {
            None => { None }
            Some(Entity(idx)) => { self.data.get(&idx) }
        }
    }

    /// Fetch a component from given entity. Do not search parent entities.
    pub fn get_local(self, Entity(idx): Entity) -> Option<&'a C> {
        self.data.get(&idx)
    }
}

/// Mutable component access.
pub struct ComponentRefMut<'a, C: 'static> {
    ecs: &'a mut Ecs,
    data: &'a mut VecMap<C>,
}

impl<'a, C: Clone> ComponentRefMut<'a, C> {
    pub fn new(ecs: &'a mut Ecs, data: &'a mut VecMap<C>) -> ComponentRefMut<'a, C> {
        ComponentRefMut {
            ecs: ecs,
            data: data,
        }
    }

    /// Fetch a component from given entity or its parent. Copy-on-write
    /// semantics: A component found on a parent entity will be copied on
    /// local entity for mutation.
    pub fn get(self, Entity(idx): Entity) -> Option<&'a mut C> {
        match find_parent(|Entity(idx)| self.data.contains_key(&idx), self.ecs, Entity(idx)) {
            None => { None }
            Some(Entity(idx2)) if idx2 == idx => { self.data.get_mut(&idx2) }
            Some(Entity(idx2)) => {
                // Copy-on-write: Make a local copy of inherited component
                // when asking for mutable access.
                let cow = self.data.get(&idx2).expect("parent component lost").clone();
                self.data.insert(idx, cow);
                self.data.get_mut(&idx)
            }
        }
    }

    /// Add a component to entity.
    pub fn insert(self, Entity(idx): Entity, comp: C) {
        self.data.insert(idx, comp);
    }

    /// Remove a component from an entity. This will make a parent entity's
    /// component visible instead, if there is one. There is currently no way
    /// to hide components present in a parent entity.
    pub fn _remove(self, Entity(idx): Entity) {
        self.data.remove(&idx);
    }
}

fn find_parent<P: Fn<(Entity,), Output=bool>>(p: P, ecs: &Ecs, e: Entity) -> Option<Entity> {
    let mut current = e;
    loop {
        if p(current) { return Some(current); }
        match ecs.parent(current) {
            Some(parent) => { current = parent; }
            None => { return None; }
        }
    }
}
