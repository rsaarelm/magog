//! Entity component system

#![deny(missing_docs)]

#[macro_use]
extern crate serde_derive;
extern crate serde;

extern crate fnv;

use std::default::Default;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::collections::{HashSet, hash_set};
use std::slice;

/// Handle for an entity in the entity component system.
///
/// The internal value is the unique identifier for the entity. No two
/// entities should get the same UID during the lifetime of the ECS.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Entity(pub usize);

/// Operations all components must support.
pub trait AnyComponent {
    /// Remove an entity's component.
    fn remove(&mut self, e: Entity);
}

/// Storage for a single component type.
#[derive(Serialize, Deserialize)]
pub struct ComponentData<C> {
    /// Component data in a densely packed vector.
    data: Vec<C>,
    /// `Entity` values that correspond to indices in `data`.
    id: Vec<Entity>,
    /// Lookup from `Entity` values to `data` and `id` indices.
    lookup: fnv::FnvHashMap<Entity, usize>,
}

impl<C> ComponentData<C> {
    /// Construct new empty `ComponentData` instance.
    pub fn new() -> ComponentData<C> {
        ComponentData {
            data: Vec::new(),
            id: Vec::new(),
            lookup: fnv::FnvHashMap::default()
        }
    }

    /// Insert a component to an entity.
    pub fn insert(&mut self, e: Entity, comp: C) {
        debug_assert!(self.data.len() == self.id.len());

        self.lookup.insert(e, self.data.len());
        self.data.push(comp);
        self.id.push(e);
    }

    /// Return whether an entity contains this component.
    pub fn contains(&self, e: Entity) -> bool {
        self.lookup.contains_key(&e)
    }

    /// Get a reference to a component only if it exists for this entity.
    pub fn get(&self, e: Entity) -> Option<&C> {
        self.lookup.get(&e).map(|&idx| &self.data[idx])
    }

    /// Get a mutable reference to a component only if it exists for this entity.
    pub fn get_mut(&mut self, e: Entity) -> Option<&mut C> {
        if let Some(idx) = self.lookup.get(&e).cloned() {
            Some(&mut self.data[idx])
        } else {
            None
        }
    }

    /// Iterate entity ids in this component.
    pub fn ent_iter(&self) -> slice::Iter<Entity> {
        self.id.iter()
    }

    /// Iterate elements in this component.
    pub fn iter(&self) -> slice::Iter<C> {
        self.data.iter()
    }

    /// Iterate mutable elements in this component.
    pub fn iter_mut(&mut self) -> slice::IterMut<C> {
        self.data.iter_mut()
    }
}

impl<C> Index<Entity> for ComponentData<C> {
    type Output = C;

    fn index<'a>(&'a self, e: Entity) -> &'a C {
        self.get(e).unwrap()
    }
}

impl<C> IndexMut<Entity> for ComponentData<C> {
    fn index_mut<'a>(&'a mut self, e: Entity) -> &'a mut C {
        self.get_mut(e).unwrap()
    }
}

impl<C> AnyComponent for ComponentData<C> {
    fn remove(&mut self, e: Entity) {
        debug_assert!(self.data.len() == self.id.len());
        if let Some(&idx) = self.lookup.get(&e) {
            debug_assert!(idx <= self.id.len());
            debug_assert!(self.id[idx] == e);

            // To keep the data compact, we do swap-remove with the last data item and update the
            // lookup on the moved item. If the component being removed isn't the last item in the
            // list, we need to reset the lookup value for the component that was moved.
            if idx != self.id.len() - 1 {
                let last_entity = self.id[self.id.len() - 1];
                self.id.swap_remove(idx);
                debug_assert!(self.id[idx] == last_entity);
                self.lookup.insert(last_entity, idx);
            } else {
                self.id.swap_remove(idx);
            }

            self.data.swap_remove(idx);
            self.lookup.remove(&e);

        }
    }
}

/// Operations for the internal component store object.
pub trait Store {
    /// Perform an operation for each component container.
    fn for_each_component<F>(&mut self, f: F) where F: FnMut(&mut AnyComponent);
}

/// Generic entity component system container
///
/// Needs to be specified with the parametrized `Store` type that has struct fields for the actual
/// components. This can be done with the `Ecs!` macro.
#[derive(Serialize, Deserialize)]
pub struct Ecs<ST> {
    next_uid: usize,
    active: HashSet<Entity>,
    store: ST,
}

impl<ST: Default + Store> Ecs<ST> {
    /// Construct a new entity component system.
    pub fn new() -> Ecs<ST> {
        Ecs {
            next_uid: 1,
            active: HashSet::default(),
            store: Default::default(),
        }
    }

    /// Create a new empty entity.
    pub fn make(&mut self) -> Entity {
        let next = self.next_uid;
        self.next_uid += 1;
        let ret = Entity(next);
        self.active.insert(ret);
        ret
    }

    /// Remove an entity from the system and clear its components.
    pub fn remove(&mut self, e: Entity) {
        self.active.remove(&e);
        self.store.for_each_component(|c| c.remove(e));
    }

    /// Return whether the system contains an entity.
    pub fn contains(&self, e: Entity) -> bool {
        self.active.contains(&e)
    }

    /// Iterate through all the active entities.
    pub fn iter(&self) -> hash_set::Iter<Entity> {
        self.active.iter()
    }
}

impl<ST> Deref for Ecs<ST> {
    type Target = ST;

    fn deref(&self) -> &ST {
        &self.store
    }
}

impl<ST> DerefMut for Ecs<ST> {
    fn deref_mut(&mut self) -> &mut ST {
        &mut self.store
    }
}

/// Entity component system builder macro.
///
/// Defines a local `Ecs` type that's parametrized with a custom component
/// store type with the component types you specify. Will also define a trait
/// `Component` which will be implemented for the component types.
#[macro_export]
macro_rules! Ecs {
    {
        // Declare the type of the (plain old data) component and the
        // identifier to use for it in the ECS.
        $($compname:ident: $comptype:ty,)+
    } => {
        mod _ecs_inner {
            // Use the enum to convert components to numbers for component bit masks etc.
            #[allow(non_camel_case_types, dead_code)]
            pub enum ComponentNum {
                $($compname,)+
            }

        }

        pub use self::_ecs_inner::ComponentNum;

        #[derive(Serialize, Deserialize)]
        pub struct _ComponentStore {
            $(pub $compname: $crate::ComponentData<$comptype>),+
        }

        impl ::std::default::Default for _ComponentStore {
            fn default() -> _ComponentStore {
                _ComponentStore {
                    $($compname: $crate::ComponentData::new()),+
                }
            }
        }

        impl $crate::Store for _ComponentStore {
            fn for_each_component<F>(&mut self, mut f: F)
                where F: FnMut(&mut $crate::AnyComponent)
            {
                $(f(&mut self.$compname as &mut $crate::AnyComponent);)+
            }
        }

        #[allow(dead_code)]
        pub fn matches_mask(ecs: &$crate::Ecs<_ComponentStore>, e: $crate::Entity, mask: u64) -> bool {
            $(if mask & (1 << ComponentNum::$compname as u8) != 0 && !ecs.$compname.contains(e) {
                return false;
            })+
            return true;
        }

        /// Common operations for ECS component value types.
        pub trait Component {
            /// Add a clone of the component value to an entity in an ECS.
            ///
            /// Can't move the component itself since we might be using this
            /// through a trait object.
            fn add_to_ecs(&self, ecs: &mut $crate::Ecs<_ComponentStore>, e: $crate::Entity);

            /// Add a clone of the component to a loadout struct.
            fn add_to_loadout(self, loadout: &mut Loadout);
        }

        $(impl Component for $comptype {
            fn add_to_ecs(&self, ecs: &mut $crate::Ecs<_ComponentStore>, e: $crate::Entity) {
                ecs.$compname.insert(e, self.clone());
            }

            fn add_to_loadout(self, loadout: &mut Loadout) {
                loadout.$compname = Some(self);
            }
        })+

        pub type Ecs = $crate::Ecs<_ComponentStore>;

        /// A straightforward representation for the complete data of an
        /// entity.
        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct Loadout {
            $(pub $compname: Option<$comptype>),+
        }

        impl ::std::default::Default for Loadout {
            fn default() -> Loadout {
                Loadout {
                    $($compname: None),+
                }
            }
        }

        #[allow(dead_code)]
        impl Loadout {
            /// Create a new blank loadout.
            pub fn new() -> Loadout { Default::default() }

            /// Get the loadout that corresponds to an existing entity.
            pub fn get(ecs: &Ecs, e: $crate::Entity) -> Loadout {
                Loadout {
                    $($compname: ecs.$compname.get(e).map(|e| e.clone())),+
                }
            }

            /// Create a new entity in the ECS with this loadout.
            pub fn make(&self, ecs: &mut Ecs) -> $crate::Entity {
                let e = ecs.make();
                $(self.$compname.as_ref().map(|c| ecs.$compname.insert(e, c.clone()));)+
                e
            }

            /// Builder method for adding a component to this loadout.
            pub fn c<C: Component>(mut self, comp: C) -> Loadout {
                comp.add_to_loadout(&mut self);
                self
            }
        }
    }
}

/// Build a component type mask to match component iteration with.
///
/// You must have ComponentNum enum from the Ecs! macro expansion in scope
/// when using this.
#[macro_export]
macro_rules! build_mask {
    ( $($compname:ident),+ ) => {
        0u64 $(| (1u64 << ComponentNum::$compname as u8))+
    }
}
