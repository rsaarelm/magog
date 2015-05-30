/*!
 * Entity component system
 */

extern crate rustc_serialize;

use std::default::{Default};
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::collections::{HashMap, HashSet};

/// Entity unique identifier type.
type Uid = i32;

/// Handle for an entity in the entity component system.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, RustcDecodable, RustcEncodable)]
pub struct Entity {
    /// Unique identifier for the entity.
    ///
    /// No two entities should get the same UID during the lifetime of the ECS.
    /// UIDs for regular entities are increasing positive integers. UIDs for
    /// prototype entities are negative integers.
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
    active: HashSet<Uid>,
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
        self.active.insert(next);
        Entity { uid: next }
    }

    /// Remove an entity from the system and clear its components.
    pub fn remove(&mut self, e: Entity) {
        self.active.remove(&e.uid);
        self.store.for_each_component(|c| c.remove(e));
    }

    /// Return whether the system contains an entity.
    pub fn contains(&self, e: Entity) -> bool {
        self.active.contains(&e.uid)
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
/// Builds a type `ComponentStore`, which can be used to parametrize an
/// `Ecs` type, with the component types you specify. Will also define a
/// trait `Component` which will be implemented for the component types.
#[macro_export]
macro_rules! ComponentStore {
    {
        // Declare the type of the (plain old data) component and the
        // identifier to use for it in the ECS.
        //
        // Component paths must be absolute.
        $($compname:ident: $comptype:ty,)+
    } => {
        mod _ecs_inner {
            // Use the enum to convert components to numbers for component bit masks etc.
            #[allow(non_camel_case_types)]
            pub enum ComponentNum {
                $($compname,)+
            }

        }

        pub struct ComponentStore {
            $($compname: ::calx_ecs::ComponentData<$comptype>),+
        }

        impl ::std::default::Default for ComponentStore {
            fn default() -> ComponentStore {
                ComponentStore {
                    $($compname: ::calx_ecs::ComponentData::new()),+
                }
            }
        }

        impl ::calx_ecs::Store for ComponentStore {
            fn for_each_component<F>(&mut self, mut f: F)
                where F: FnMut(&mut ::calx_ecs::AnyComponent)
            {
                $(f(&mut self.$compname as &mut ::calx_ecs::AnyComponent);)+
            }
        }

        /// Common operations for ECS component value types.
        pub trait Component {
            /// Return a type identifier for the component type.
            fn type_num() -> u16;

            /// Add the component value to an entity in an ECS.
            fn add_to(self, ecs: &mut ::calx_ecs::Ecs<ComponentStore>, e: ::calx_ecs::Entity);
        }

        $(impl Component for $comptype {
            fn type_num() -> u16 {
                _ecs_inner::ComponentNum::$compname as u16
            }

            fn add_to(self, ecs: &mut ::calx_ecs::Ecs<ComponentStore>, e: ::calx_ecs::Entity) {
                ecs.$compname.insert(e, self);
            }
        })+
    }
}
