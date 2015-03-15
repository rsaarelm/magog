use std::collections::HashMap;
use ecs::Ecs;
use entity::Entity;

/// Immutable component access.
pub struct ComponentRef<'a, C: 'static> {
    ecs: &'a Ecs,
    data: &'a HashMap<usize, Option<C>>,
}

impl<'a, C> ComponentRef<'a, C> {
    pub fn new(ecs: &'a Ecs, data: &'a HashMap<usize, Option<C>>) -> ComponentRef<'a, C> {
        ComponentRef {
            ecs: ecs,
            data: data,
        }
    }

    /// Fetch a component from given entity or its parent.
    pub fn get(self, Entity(idx): Entity) -> Option<&'a C> {
        match find_parent(|Entity(idx)| self.data.contains_key(&idx), self.ecs, Entity(idx)) {
            None => { None }
            Some(Entity(idx)) => { self.data.get(&idx).expect("missing component").as_ref() }
        }
    }

    /// Fetch a component from given entity. Do not search parent entities.
    pub fn get_local(self, Entity(idx): Entity) -> Option<&'a C> {
        self.data.get(&idx).map_or(None, |x| x.as_ref())
    }
}

/// Mutable component access.
pub struct ComponentRefMut<'a, C: 'static> {
    ecs: &'a mut Ecs,
    /// Some(c) indicates the presence of a local component, None indicates
    /// that there is no local component and that the component will not be
    /// searched from the parent entity either. If the value is not present,
    /// the component will be searched from a entity object if one exists.
    data: &'a mut HashMap<usize, Option<C>>,
}

impl<'a, C: Clone> ComponentRefMut<'a, C> {
    pub fn new(ecs: &'a mut Ecs, data: &'a mut HashMap<usize, Option<C>>) -> ComponentRefMut<'a, C> {
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
            Some(Entity(idx2)) if idx2 == idx => {
                self.data.get_mut(&idx2).expect("missing component").as_mut()
            }
            Some(Entity(idx2)) => {
                // Copy-on-write: Make a local copy of inherited component
                // when asking for mutable access.
                let cow = self.data.get(&idx2)
                    .expect("parent component lost").as_ref()
                    .expect("missing component").clone();
                self.data.insert(idx, Some(cow));
                self.data.get_mut(&idx).expect("missing component").as_mut()
            }
        }
    }

    /// Add a component to entity.
    pub fn insert(self, Entity(idx): Entity, comp: C) {
        self.data.insert(idx, Some(comp));
    }

    /// Clear a component from an entity. This will make a parent entity's
    /// component visible instead, if there is one.
    pub fn clear(self, Entity(idx): Entity) {
        self.data.remove(&idx);
    }

    /// Make a component not show up on an entity even if it is present in the
    /// parent entity. Hiding will be reset if the component is cleared.
    pub fn hide(self, Entity(idx): Entity) {
        self.data.insert(idx, None);
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
