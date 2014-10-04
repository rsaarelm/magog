use std::intrinsics::TypeId;
use std::collections::hashmap::HashMap;
use std::any::{Any, AnyRefExt, AnyMutRefExt};

pub type Uid = u64;

#[deriving(Clone, PartialEq, Eq)]
pub struct Entity(Uid);

/// Entity component system
pub struct Ecs {
    // XXX: This is much less efficient than it could be. A serious
    // implementation would use unboxed vecs for the components and would
    // provide lookup methods faster than a HashMap find to access the
    // components
    components: HashMap<TypeId, Vec<Option<Box<Any + 'static>>>>,

    next_idx: uint,
    reusable_idxs: Vec<uint>,
    uids: Vec<Option<Uid>>,
    next_uid: Uid,
}

impl Ecs {
    pub fn new() -> Ecs {
        Ecs {
            components: HashMap::new(),
            next_idx: 0,
            reusable_idxs: vec![],
            uids: vec![],
            next_uid: 1,
        }
    }

    pub fn new_entity(&self) -> Entity {
        unimplemented!();
    }

    pub fn delete_entity(&self, e: Entity) {
        unimplemented!();
    }
}
