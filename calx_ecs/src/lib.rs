/*!
 * Entity component system
 */

extern crate rustc_serialize;

use std::collections::{HashMap};

/// Entity index type.
pub type Idx = u32;
/// Entity unique identifier type.
pub type Uid = i32;

/// Handle for an entity in the entity component system.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, RustcDecodable, RustcEncodable)]
pub struct Entity {
    /// Internal index in the ECS.
    pub idx: Idx,
    /// Unique identifier for the entity.
    ///
    /// No two entities should get the same UID during the lifetime of the ECS.
    /// UIDs for regular entities are increasing positive integers. UIDs for
    /// prototype entities are negative integers.
    pub uid: Uid,
}

impl Entity {
    /// Return whether this is a prototype entity.
    pub fn is_prototype(self) -> bool {
        self.uid < 0
    }
}

/// Immutable component accessor.
pub struct CompRef<'a, C: 'static> {
    parents: &'a HashMap<Idx, Entity>,
    active: &'a HashMap<Idx, Uid>,
    data: &'a HashMap<Idx, Option<C>>,
}

impl<'a, C> CompRef<'a, C> {
    pub fn new(parents: &'a HashMap<Idx, Entity>, active: &'a HashMap<Idx, Uid>, data: &'a HashMap<Idx, Option<C>>) -> CompRef<'a, C> {
        CompRef {
            parents: parents,
            active: active,
            data: data,
        }
    }

    /// Fetch a component from given entity or its parent.
    pub fn get(&'a self, e: Entity) -> Option<&'a C> {
        self.check_uid(e);
        let mut current = e;
        loop {
            match self.data.get(&current.idx) {
                Some(&None) => { return None; }
                Some(x) => { return x.as_ref(); }
                None => {
                    if let Some(&p) = self.parents.get(&current.idx) {
                        current = p;
                    } else {
                        return None;
                    }
                }
            }
        }
    }

    /// Fetch a component from given entity. Do not search parent entities.
    pub fn get_local(&'a self, e: Entity) -> Option<&'a C> {
        self.check_uid(e);
        self.data.get(&e.idx).map_or(None, |x| x.as_ref())
    }

    #[inline]
    fn check_uid(&'a self, e: Entity) {
        assert!(self.active.get(&e.idx) == Some(&e.uid), "Stale entity handle");
    }
}


/// Mutable component accessor.
pub struct CompRefMut<'a, C: 'static> {
    parents: &'a HashMap<Idx, Entity>,
    active: &'a HashMap<Idx, Uid>,
    data: &'a mut HashMap<Idx, Option<C>>,
}

impl<'a, C: Clone> CompRefMut<'a, C> {
    pub fn new(parents: &'a HashMap<Idx, Entity>, active: &'a HashMap<Idx, Uid>, data: &'a mut HashMap<Idx, Option<C>>) -> CompRefMut<'a, C> {
        CompRefMut {
            parents: parents,
            active: active,
            data: data,
        }
    }

    /// Fetch a component from given entity. Copy-on-write from parent
    /// if found on parent but not locally.
    pub fn get(&'a mut self, e: Entity) -> Option<&'a mut C> {
        self.check_uid(e);
        if let Some(c) = self.cow_source(e) {
            let comp = self.data.get(&c.idx).unwrap().as_ref().unwrap().clone();
            self.data.insert(e.idx, Some(comp));
        }

        self.data.get_mut(&e.idx).map_or(None, |x| x.as_mut())
    }

    fn cow_source(&'a self, e: Entity) -> Option<Entity> {
        let mut current = e;
        loop {
            match self.data.get(&current.idx) {
                Some(&None) => { return None; }
                Some(_) => {
                    if current != e { return Some(current); } else { return None; }
                }
                None => {
                    if let Some(&p) = self.parents.get(&current.idx) {
                        current = p;
                    } else {
                        return None;
                    }
                }
            }
        }
    }

    /// Insert a component
    pub fn insert(&'a mut self, e: Entity, comp: C) {
        self.check_uid(e);
        self.data.insert(e.idx, Some(comp));
    }

    /// Clear a component from  the entity. Will make parent prototype's
    /// version visible if there is one.
    pub fn clear(&'a mut self, e: Entity) {
        self.check_uid(e);
        self.data.remove(&e.idx);
    }

    /// Make the component locally invisible even if the parent
    /// prototype has it.
    pub fn hide(&'a mut self, e: Entity) {
        self.check_uid(e);
        self.data.insert(e.idx, None);
    }

    #[inline]
    fn check_uid(&'a self, e: Entity) {
        assert!(self.active.get(&e.idx) == Some(&e.uid), "Stale entity handle");
    }
}

/// Opaque identifier for a component type
pub struct CompId(pub u16);

/// Common operations for all component types
pub trait Component<E> {
    /// Create an uniform syntax for attaching components to
    /// entities to allow a fluent API for constructing
    /// prototypes.
    fn add_to(self, ecs: &mut E, e: Entity);

    /// Get the ID for this component.
    fn id() -> CompId;
}


// XXX: Copy-pasted macro from calx_macros since using imported macros
// from other crates inside code defined in a local macro doesn't really
// work out.
#[doc(hidden)]
#[macro_export]
macro_rules! __count_exprs {
    () => { 0 };
    ($e:expr) => { 1 };
    ($e:expr, $($es:expr),+) => { 1 + __count_exprs!($($es),*) };
}

/// Entity component system builder macro.
///
/// Builds a type `Ecs` with the component types you specify. See the unit tests
/// for usage examples.
#[macro_export]
macro_rules! Ecs {
    {
        // Declare the type of the (plain old data) component and the
        // identifier to use for it in the ECS.
        //
        // Component paths must be absolute.
        $($compname:ident: $comptype:ty,)+
    } => {
        mod _ecs_inner {
            use ::std::collections::{HashMap};
            use ::calx_ecs::{Idx, Uid, Entity, CompRef, CompRefMut, CompId, Component};

            // Use the enum to convert components to numbers for component bit masks etc.
            #[allow(non_camel_case_types)]
            enum ComponentNum {
                $($compname,)+
            }


            $(impl Component<Ecs> for $comptype {
                fn add_to(self, ecs: &mut Ecs, e: Entity) { ecs.mu().$compname().insert(e, self); }

                fn id() -> CompId {
                    CompId(ComponentNum::$compname as u16)
                }
            })+

            // Don't create noise if the user doesn't use every method.
            /// Entity component system main container.
            #[derive(RustcEncodable, RustcDecodable)]
            pub struct Ecs {
                /// Next positional index
                next_idx: Idx,

                /// Queue for reusable indices
                reusable_idxs: Vec<Idx>,

                /// Next unique identifier
                next_entity_uid: Uid,

                /// Next unique identifier for prototypes
                ///
                /// Prototypes use negative UID values.
                next_prototype_uid: Uid,

                /// Live entities
                // TODO: Use BitVec when it's stable and serializable.
                active: HashMap<Idx, Uid>,

                /// Parent entity table
                // TODO: Use BitVec when it's stable and serializable.
                parents: HashMap<Idx, Entity>,

                // Component value storage.
                //
                // An explicit "None" value for a component means it's
                // considered not present even if there's a parent entity that
                // does have it.
                $($compname: HashMap<Idx, Option<$comptype>>,)+
            }

            impl Ecs {
                pub fn new() -> Ecs {
                    Ecs {
                        next_idx: 0,
                        reusable_idxs: Vec::new(),
                        // Don't use uid 0.
                        next_entity_uid: 1,
                        next_prototype_uid: -1,
                        active: HashMap::new(),
                        parents: HashMap::new(),

                        $($compname: HashMap::new(),)+
                    }
                }

                /// Make a new entity.
                pub fn make(&mut self, parent: Option<Entity>) -> Entity {
                    let uid = self.next_entity_uid;
                    self.next_entity_uid += 1;
                    self._make(uid, parent)
                }

                /// Make a new entity prototype.
                pub fn make_prototype(&mut self, parent: Option<Entity>) -> Entity {
                    let uid = self.next_prototype_uid;
                    self.next_prototype_uid -= 1;
                    self._make(uid, parent)
                }

                fn _make(&mut self, uid: Uid, parent: Option<Entity>) -> Entity {
                    let idx = if let Some(idx) = self.reusable_idxs.pop() { idx } else {
                        self.next_idx += 1;
                        self.next_idx - 1
                    };

                    self.active.insert(idx, uid);
                    let ret = Entity { idx: idx, uid: uid };

                    if let Some(parent) = parent { self.set_parent(ret, Some(parent)); }

                    ret
                }

                /// Return whether a given entity exists in the ECS.
                pub fn contains(&self, e: Entity) -> bool {
                    match self.active.get(&e.idx) {
                        // The entity slot might have been reused, verify that the
                        // UID is still same.
                        Some(&stored_uid) => e.uid == stored_uid,
                        _ => false
                    }
                }

                fn remove_internal(&mut self, e: Entity) {
                    assert!(self.contains(e), "Deleting an entity not contained in ECS");
                    assert!(!e.is_prototype(), "Prototype entities cannot be deleted.");
                    self.parents.remove(&e.idx);
                    self.reusable_idxs.push(e.idx);
                    self.active.remove(&e.idx);
                }


                /// Set or unset a prototype parent for an entity.
                pub fn set_parent(&mut self, e: Entity, parent: Option<Entity>) {
                    if let Some(parent) = parent {
                        assert!(parent.is_prototype(), "Trying to assign non-prototype parent entity");
                        assert!(self.contains(parent), "Parent of entity not present in ECS");
                        self.parents.insert(e.idx, parent);
                    } else {
                        self.parents.remove(&e.idx);
                    }
                }

                /// Get the prototype parent of an entity if there is one.
                pub fn get_parent(&self, e: Entity) -> Option<Entity> {
                    self.parents.get(&e.idx).map(|&p| p)
                }

                /// Return the non-prototype entity with the lowest idx
                ///
                /// A building block for making an entity iterator.
                pub fn first_entity(&self) -> Option<Entity> {
                    self._next_entity(0)
                }

                /// Return the next non-prototype entity in idx order.
                ///
                /// A building block for making an entity iterator.
                pub fn next_entity(&self, prev: Entity) -> Option<Entity> {
                    self._next_entity(prev.idx + 1)
                }

                fn _next_entity(&self, min_idx: Idx) -> Option<Entity> {
                    if self.active.is_empty() { return None; }

                    for i in min_idx..self.next_idx {
                        if let Some(&uid) = self.active.get(&i) {
                            if uid >= 0 {
                                return Some(Entity { idx: i, uid: uid });
                            }
                        }
                    }

                    return None;
                }

                /// Return the largest unique identifier in use.
                ///
                /// Will return 0 (which is never used as an UID) if no
                /// non-prototype entities have yet been created.
                ///
                /// Use this to set up an entity traversal that won't return
                /// entities with UIDs larger than what the largest UID was when
                /// the traversal started. (Eg. iterate all currently existing
                /// entities in a game update loop, but don't iterate any new
                /// entities that were generated during the update procedure of
                /// some existing current entity)
                pub fn largest_uid(&self) -> Uid { self.next_entity_uid - 1 }

                /// Return whether an entity has a component given the component's Id value.
                pub fn has_indexed_component(&self, id: CompId, e: Entity) -> bool {
                    static TBL: [ComponentNum; __count_exprs!($($compname),+)] = [
                        $(ComponentNum::$compname,)+
                    ];

                    let id = id.0 as usize;
                    if id >= TBL.len() { return false; }
                    match TBL[id] {
                        $(ComponentNum::$compname =>
                          self.$compname().get(e).is_some(),
                        )+
                    }
                }

                /// Remove an entity and all its components.
                pub fn remove(&mut self, e: Entity) {
                    $(
                    self.mu().$compname().clear(e);
                    )+

                    self.remove_internal(e);
                }

                $(
                /// Get immutable accessor to $compname.
                pub fn $compname<'a>(&'a self) -> CompRef<'a, $comptype> {
                    CompRef::new(&self.parents, &self.active, &self.$compname)
                })+

                /// Get mutable component accessor structure.
                pub fn mu<'a>(&'a mut self) -> EcsMutHandle<'a> {
                    EcsMutHandle { ecs: self }
                }
            }

            /// Mutable component accessor structure.
            pub struct EcsMutHandle<'a> {
                ecs: &'a mut Ecs,
            }

            impl<'a> EcsMutHandle<'a> {
                // Mutable accessors
                $(
                /// Get mutable accessor to $compname.
                pub fn $compname<'b>(&'b mut self) -> CompRefMut<'b, $comptype> {
                    CompRefMut::new(&self.ecs.parents, &self.ecs.active, &mut self.ecs.$compname)
                })+
            }

            /// A helper struct for fluently adding components to an entity
            pub struct Build<'a> {
                ecs: &'a mut Ecs,
                e: Entity,
            }

            impl<'a> Build<'a> {
                /// Start building a prototype entity.
                pub fn prototype(ecs: &'a mut Ecs, parent: Option<Entity>) -> Build<'a> {
                    let e = ecs.make_prototype(parent);
                    Build {
                        ecs: ecs,
                        e: e,
                    }
                }

                /// Start building a regular entity.
                pub fn entity(ecs: &'a mut Ecs, parent: Option<Entity>) -> Build<'a> {
                    let e = ecs.make(parent);
                    Build {
                        ecs: ecs,
                        e: e,
                    }
                }

                /// Add a component to the entity being built.
                pub fn c<C: Component<Ecs>>(mut self, comp: C) -> Build<'a> {
                    comp.add_to(self.ecs, self.e);
                    self
                }

                /// Return the built entity.
                pub fn e(self) -> Entity { self.e }
            }
        }

        pub use self::_ecs_inner::{Ecs, Build};
    }
}
