/*!
 * Entity component system
 */

extern crate rustc_serialize;

use std::default::{Default};
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::collections::{HashMap, HashSet};
use std::collections::hash_set;

/// Entity unique identifier type.
type Uid = i32;

/// Handle for an entity in the entity component system.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, RustcDecodable, RustcEncodable)]
pub struct Entity {
    /// Unique identifier for the entity.
    ///
    /// No two entities should get the same UID during the lifetime of the ECS.
    /// UIDs for regular entities are increasing positive integers.
    uid: Uid,
}

pub trait AnyComponent {
    /// Remove an entity's component.
    fn remove(&mut self, e: Entity);
}

/// Storage for a single component type.
#[derive(RustcEncodable, RustcDecodable)]
pub struct ComponentData<C> {
    // TODO: Add reused index fields to entities and use VecMap with the
    // index field instead of HashMap with the UID here for more
    // efficient access.
    data: HashMap<Uid, C>,
}

impl<C> ComponentData<C> {
    pub fn new() -> ComponentData<C> {
        ComponentData {
            data: HashMap::new()
        }
    }

    /// Insert a component to an entity.
    pub fn insert(&mut self, e: Entity, comp: C) {
        self.data.insert(e.uid, comp);
    }

    /// Return whether an entity contains this component.
    pub fn contains(&self, e: Entity) -> bool {
        self.data.contains_key(&e.uid)
    }
}

impl<C> Index<Entity> for ComponentData<C> {
    type Output = C;

    fn index<'a>(&'a self, e: Entity) -> &'a C {
        self.data.get(&e.uid).unwrap()
    }
}

impl<C> IndexMut<Entity> for ComponentData<C> {
    fn index_mut<'a>(&'a mut self, e: Entity) -> &'a mut C {
        self.data.get_mut(&e.uid).unwrap()
    }
}

impl<C> AnyComponent for ComponentData<C> {
    fn remove(&mut self, e: Entity) {
        self.data.remove(&e.uid);
    }
}

pub trait Store {
    /// Perform an operation for each component container.
    fn for_each_component<F>(&mut self, f: F)
        where F: FnMut(&mut AnyComponent);
}


#[derive(RustcEncodable, RustcDecodable)]
pub struct Ecs<S> {
    next_uid: Uid,
    active: HashSet<Entity>,
    store: S,
}

impl<S: Default+Store> Ecs<S> {
    pub fn new() -> Ecs<S> {
        Ecs {
            next_uid: 1,
            active: HashSet::new(),
            store: Default::default(),
        }
    }

    /// Create a new empty entity.
    pub fn make(&mut self) -> Entity {
        let next = self.next_uid;
        self.next_uid += 1;
        let ret = Entity { uid: next };
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

    pub fn iter(&self) -> hash_set::Iter<Entity> {
        self.active.iter()
    }
}

impl<S> Deref for Ecs<S> {
    type Target = S;

    fn deref(&self) -> &S { &self.store }
}

impl<S> DerefMut for Ecs<S> {
    fn deref_mut(&mut self) -> &mut S {&mut self.store}
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

#[derive(RustcEncodable, RustcDecodable)]
        pub struct _ComponentStore {
            $(pub $compname: ::calx_ecs::ComponentData<$comptype>),+
        }

        impl ::std::default::Default for _ComponentStore {
            fn default() -> _ComponentStore {
                _ComponentStore {
                    $($compname: ::calx_ecs::ComponentData::new()),+
                }
            }
        }

        impl ::calx_ecs::Store for _ComponentStore {
            fn for_each_component<F>(&mut self, mut f: F)
                where F: FnMut(&mut ::calx_ecs::AnyComponent)
            {
                $(f(&mut self.$compname as &mut ::calx_ecs::AnyComponent);)+
            }
        }

#[allow(dead_code)]
        pub fn matches_mask(ecs: &::calx_ecs::Ecs<_ComponentStore>, e: ::calx_ecs::Entity, mask: u64) -> bool {
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
            fn add_to(&self, ecs: &mut ::calx_ecs::Ecs<_ComponentStore>, e: ::calx_ecs::Entity);
        }

        $(impl Component for $comptype {
            fn add_to(&self, ecs: &mut ::calx_ecs::Ecs<_ComponentStore>, e: ::calx_ecs::Entity) {
                ecs.$compname.insert(e, self.clone());
            }
        })+

        pub type Ecs = ::calx_ecs::Ecs<_ComponentStore>;
    }
}

/// Build a vector of Component trait objects from component values.
///
/// Use to set up prototype templates for entities.
#[macro_export]
macro_rules! loadout {
    [ $($comp:expr),+ ] => {
        vec![
            $(Box::new($comp) as Box<Component>),+
        ]
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
