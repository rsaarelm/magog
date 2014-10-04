use std::intrinsics::TypeId;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::fmt::{Formatter, Show, Result};
use std::collections::hashmap::HashMap;
use std::any::{Any, AnyRefExt, AnyMutRefExt};
use std::hash::Hash;
use std::hash::sip::SipState;
use std::mem;
use uuid::Uuid;

/// Toplevel container of the entity component system.
pub struct WorldBase<T> {
    data: Rc<RefCell<WorldData<T>>>,
}

impl<T> Clone for WorldBase<T> {
    fn clone(&self) -> WorldBase<T> {
        WorldBase { data: self.data.clone() }
    }
}

impl<T: SystemBase> WorldBase<T> {
    /// Create a world coupled with an application specific system object.
    pub fn new(master_system: T) -> WorldBase<T> {
        let ret = WorldBase {
            data: Rc::new(RefCell::new(WorldData::new(master_system))),
        };
        ret.data.borrow_mut().master_system.register(&ret);
        ret
    }

    /// Wrap an EntityId struct into an entity handle that has a backreference
    /// to the world object.
    fn wrap(&self, id: EntityId) -> EntityBase<T> {
        EntityBase {
            world: self.data.downgrade(),
            id: id
        }
    }

    /// Create a new entity bound to this world.
    pub fn new_entity(&mut self) -> EntityBase<T> {
        let id = EntityId {
            idx: self.data.borrow_mut().get_idx(),
            uuid: Uuid::new_v4(),
        };
        self.data.borrow_mut().register_new_entity(id);
        let ret = self.wrap(id);
        self.data.borrow_mut().master_system.added(&ret);
        ret
    }

    pub fn delete_entity(&mut self, e: &EntityBase<T>) {
        self.data.borrow_mut().delete_entity(e);
    }

    pub fn system<'a>(&'a self) -> &'a T {
        unsafe { mem::transmute(&self.data.borrow().master_system) }
    }

    pub fn system_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { mem::transmute(&self.data.borrow_mut().master_system) }
    }

    pub fn find_entity(&self, uuid: &Uuid) -> Option<EntityBase<T>> {
        match self.data.borrow().uuids.as_slice().position_elem(&Some(*uuid)) {
            Some(idx) => Some(self.wrap(EntityId { uuid: *uuid, idx: idx })),
            None => None
        }
    }

    /// Return an iterator for the entities. The iterator will not be
    /// invalidated if entities are added or removed during iteration.
    ///
    /// XXX: It is currently unspecified whether entities added during
    /// iteration will show up in the iteration or not.
    pub fn entities(&self) -> Entities<T> {
        Entities {
            idx: 0,
            world: self.data.downgrade(),
        }
    }
}

pub struct Entities<T> {
    idx: uint,
    world: Weak<RefCell<WorldData<T>>>,
}

impl<T> Iterator<EntityBase<T>> for Entities<T> {
    fn next(&mut self) -> Option<EntityBase<T>> {
        let w = self.world.upgrade().unwrap();
        loop {
            let uuids = &w.borrow().uuids;
            if self.idx >= uuids.len() { return None; }
            let (idx, uuid) = (self.idx, uuids[self.idx]);
            self.idx += 1;
            match uuid {
                Some(uuid) => return Some(EntityBase {
                    world: self.world.clone(),
                    id: EntityId {
                        uuid: uuid,
                        idx: idx
                    }
                }),
                _ => ()
            }
        }
    }
}


/// Callback interface for application data connected to the component system
/// world.
pub trait SystemBase {
    /// Callback for initially attaching the system to a world.
    fn register(&mut self, world: &WorldBase<Self>);
    /// Callback for adding a new entity to world.
    fn added(&mut self, e: &EntityBase<Self>);
    /// Callback for a component being added or removed in an entity. Note that
    /// this is not called when the data of an existing component is written
    /// to, only if the component is replaced or removed.
    fn changed<C>(&mut self, e: &EntityBase<Self>, component: Option<&C>);
    /// Callback for an entity being deleted.
    fn deleted(&mut self, e: &EntityBase<Self>);
}

pub struct EntityBase<T> {
    world: Weak<RefCell<WorldData<T>>>,
    id: EntityId,
}

impl <T> Hash for EntityBase<T> {
    fn hash(&self, state: &mut SipState) {
        self.id.hash(state);
    }
}

impl <T> PartialEq for EntityBase<T> {
    fn eq(&self, other: &EntityBase<T>) -> bool {
        self.id.eq(&other.id)
    }
}

impl <T> Eq for EntityBase<T> {}

impl<T> Clone for EntityBase<T> {
    fn clone(&self) -> EntityBase<T> {
        EntityBase { world: self.world.clone(), id: self.id }
    }
}

impl<T> Show for EntityBase<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.id)
    }
}

impl<T: SystemBase> EntityBase<T> {
    /// Add or reset a component in the entity. Causes a "changed" call to the
    /// world system.
    ///
    /// Due to the system used to identify components, the component type
    /// needs to be a 'static one that doesn't include any reference values.
    /// Components are generally expected to be very plain data, like a
    /// database record.
    pub fn set_component<C: 'static+Clone>(&mut self, comp: C) {
        self.world.upgrade().unwrap().borrow_mut().deref_mut()
            .set_component(self, Some(comp));
    }

    /// Remove a component from the entity. Causes a "changed" call to the
    /// world system.
    pub fn remove_component<C: 'static+Clone>(&mut self) {
        let comp : Option<C> = None;
        self.world.upgrade().unwrap().borrow_mut().deref_mut()
            .set_component(self, comp);
    }

    /// Delete the entity from the world. Causes a "deleted" call to the world
    /// system.
    pub fn delete(&mut self) {
        self.world.upgrade().unwrap().borrow_mut().deref_mut()
            .delete_entity(self);
    }

    /// Query whether the entity has a type of component.
    pub fn has<'a, C: 'static>(&self) -> bool {
        self.world.upgrade().unwrap().borrow()
            .comp_ref::<'a, C>(self.id).is_some()
    }

    /// Get an access proxy to the entity's component if the entity has that
    /// component.
    pub fn into<C: 'static>(&self) -> Option<CompProxyMut<T, C>> {
        if self.has::<C>() {
            Some(CompProxyMut {
                world: self.world.clone(),
                id: self.id
            })
        } else {
            None
        }
    }

    /// Return unique identifier for this entity.
    pub fn uuid(&self) -> Uuid {
        self.id.uuid
    }

    /// Return a handle to the world.
    pub fn world(&self) -> WorldBase<T> {
        WorldBase { data: self.world.upgrade().unwrap() }
    }
}

/// Access proxy to entity components. The implementation RefCell (mutable)
/// borrow internally, so using this carelessly may lead to runtime borrow
/// errors. You should generally keep access to the concrete entity data short
/// in scope and low in the call stack to prevent unexpected borrow failures.
pub struct CompProxyMut<T, C> {
    world: Weak<RefCell<WorldData<T>>>,
    id: EntityId,
}

impl<T: SystemBase, C: 'static> Deref<C> for CompProxyMut<T, C> {
    fn deref<'a>(&'a self) -> &'a C {
        self.world.upgrade().unwrap().borrow()
            .comp_ref::<'a, C>(self.id).unwrap()
    }
}

impl<T: SystemBase, C: 'static> DerefMut<C> for CompProxyMut<T, C> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut C {
        self.world.upgrade().unwrap().borrow_mut()
            .comp_ref_mut::<'a, C>(self.id).unwrap()
    }
}

#[deriving(PartialEq, Clone, Hash, Show)]
struct EntityId {
    uuid: Uuid,
    idx: uint,
}

struct WorldData<T> {
    // XXX: This is much less efficient than it could be. A serious
    // implementation would use unboxed vecs for the components and would
    // provide lookup methods faster than a HashMap find to access the
    // components
    components: HashMap<TypeId, Vec<Option<Box<Any + 'static>>>>,

    next_idx: uint,
    reusable_idxs: Vec<uint>,
    uuids: Vec<Option<Uuid>>,

    master_system: T,
}

impl<T: SystemBase> WorldData<T> {
    fn new(master_system: T) -> WorldData<T> {
        WorldData {
            components: HashMap::new(),
            next_idx: 0,
            reusable_idxs: vec!(),
            uuids: vec!(),
            master_system: master_system,
        }
    }

    fn register_new_entity(&mut self, id: EntityId) {
        assert!(self.uuids.len() <= id.idx || self.uuids[id.idx].is_none(),
            "New entity clobbering existing one");

        while self.uuids.len() < (id.idx + 1) { self.uuids.push(None); }
        *self.uuids.get_mut(id.idx) = Some(id.uuid);
    }

    fn delete_entity(&mut self, e: &EntityBase<T>) {
        self.uuids.as_mut_slice()[e.id.idx] = None;

        for (_, c) in self.components.iter_mut() {
            if c.len() > e.id.idx {
                c.as_mut_slice()[e.id.idx] = None;
            }
        }

        self.reusable_idxs.push(e.id.idx);

        self.master_system.deleted(e);
    }

    /// Return the next unused reusable entity id.
    fn get_idx(&mut self) -> uint {
        match self.reusable_idxs.pop() {
            None => {
                let ret = self.next_idx;
                self.next_idx += 1;
                ret
            }
            Some(idx) => idx
        }
    }

    fn set_component<C: 'static+Clone>(&mut self, e: &EntityBase<T>, comp: Option<C>) {
        let type_id = TypeId::of::<C>();
        // We haven't seen this kind of component yet.
        if !self.components.contains_key(&type_id) {
            self.components.insert(type_id, vec!());
        }

        let bin = self.components.find_mut(&type_id).unwrap();

        // XXX: grow_set didn't work.
        while bin.len() <= e.id.idx { bin.push(None); }
        bin.as_mut_slice()[e.id.idx] = match comp {
            None => None,
            Some(c) => Some(box c as Box<Any + 'static>),
        };

        match bin.as_mut_slice()[e.id.idx] {
            None => self.master_system.changed::<C>(e, None),
            Some(ref c) => self.master_system.changed(e, Some(&c)),
        };
    }

    fn comp_ref<'a, C: 'static>(
        &self, id: EntityId) -> Option<&'a C> {
        let type_id = TypeId::of::<C>();
        match self.components.find(&type_id) {
            None => { None }
            Some(bin) => {
                if id.idx < bin.len() {
                    match bin[id.idx] {
                        Some(ref c) => { unsafe { Some(mem::transmute(c.downcast_ref::<C>().unwrap())) } }
                        None => None
                    }
                } else {
                    None
                }
            }
        }
    }

    fn comp_ref_mut<'a, C: 'static>(
        &mut self, id: EntityId) -> Option<&'a mut C> {
        let type_id = TypeId::of::<C>();
        match self.components.find_mut(&type_id) {
            None => { None }
            Some(bin) => {
                if id.idx < bin.len() {
                    match bin.get_mut(id.idx) {
                        &Some(ref mut c) => {
                            unsafe { Some(mem::transmute(c.downcast_mut::<C>().unwrap())) }
                        }
                        &None => None
                    }
                } else {
                    None
                }
            }
        }
    }
}

