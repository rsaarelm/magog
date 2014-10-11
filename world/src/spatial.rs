use std::collections::hashmap::{HashMap};
use entity::{Entity};
use serialize::{Decodable, Decoder, Encodable, Encoder};
use location::{Location};

/// Spatial index for game entities
pub struct Spatial {
    loc_to_entities: HashMap<Location, Vec<Entity>>,
    entity_to_loc: HashMap<Entity, Location>,
}

impl Spatial {
    pub fn new() -> Spatial {
        Spatial {
            loc_to_entities: HashMap::new(),
            entity_to_loc: HashMap::new(),
        }
    }

    /// Insert an entity into space.
    pub fn insert(&mut self, e: Entity, loc: Location) {
        if self.entity_to_loc.contains_key(&e) {
            self.remove(e);
        }

        self.entity_to_loc.insert(e, loc);
        match self.loc_to_entities.find_mut(&loc) {
            Some(v) => { v.push(e); return; }
            _ => ()
        };
        // Didn't return above, that means this location isn't indexed
        // yet and needs a brand new container. (Can't do this in match
        // block because borrows.)
        self.loc_to_entities.insert(loc, vec![e]);
    }

    /// Remove an entity from the space.
    pub fn remove(&mut self, e: Entity) {
        if !self.entity_to_loc.contains_key(&e) { return; }

        let loc = *self.entity_to_loc.find(&e).unwrap();
        self.entity_to_loc.remove(&e);
        {
            let v = self.loc_to_entities.find_mut(&loc).unwrap();
            assert!(v.len() > 0);
            if v.len() > 1 {
                // More than one entity present, remove this one, keep the
                // rest.
                let idx = v.as_slice().position_elem(&e).unwrap();
                v.swap_remove(idx);
                return;
            } else {
                // This was the only entity in the location.
                // Drop the entry for this location from the index.
                // (Need to drop out of scope for borrows reasons)
                assert!((*v)[0] == e);
            }
        }
        // We only end up here if we need to clear the container for the
        // location.
        self.loc_to_entities.remove(&loc);
    }

    /// List entities at a location.
    pub fn entities_at(&self, loc: Location) -> Vec<Entity> {
        match self.loc_to_entities.find(&loc) {
            None => vec![],
            Some(v) => v.clone(),
        }
    }

    /// Return the location of an entity if the entity is present in the space.
    pub fn get(&self, e: Entity) -> Option<Location> {
        self.entity_to_loc.find(&e).map(|&loc| loc)
    }

    /// Flatten to an easily serializable vector.
    fn dump(&self) -> Vec<Elt> {
        let mut ret = vec![];
        for (&e, &loc) in self.entity_to_loc.iter() {
            ret.push(Elt(e, loc));
        }
        ret
    }

    /// Construct from the serialized vector.
    fn slurp(dump: Vec<Elt>) -> Spatial {
        let mut ret = Spatial::new();

        for &Elt(e, loc) in dump.iter() {
            ret.insert(e, loc);
        }
        ret
    }
}

#[deriving(Clone, Decodable, Encodable)]
struct Elt(Entity, Location);

impl<E, D:Decoder<E>> Decodable<D, E> for Spatial {
    fn decode(d: &mut D) -> Result<Spatial, E> {
        Ok(Spatial::slurp(try!(Decodable::decode(d))))
    }
}

impl<E, S:Encoder<E>> Encodable<S, E> for Spatial {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        self.dump().encode(s)
    }
}
