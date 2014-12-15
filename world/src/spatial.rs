use std::collections::{HashMap, VecMap};
use entity::{Entity};
use serialize::{Decodable, Decoder, Encodable, Encoder};
use location::{Location};
use self::Place::*;

/// Entities can be placed either on open locations or inside other entities.
/// A sum type will represent this nicely.
#[deriving(Eq, PartialEq, Clone, Hash, Show, Encodable, Decodable)]
pub enum Place {
    At(Location),
    In(Entity),
}

/// Spatial index for game entities
pub struct Spatial {
    place_to_entities: HashMap<Place, Vec<Entity>>,
    entity_to_place: VecMap<Place>,
}

impl Spatial {
    pub fn new() -> Spatial {
        Spatial {
            place_to_entities: HashMap::new(),
            entity_to_place: VecMap::new(),
        }
    }

    fn insert(&mut self, Entity(idx): Entity, p: Place) {
        if self.entity_to_place.contains_key(&idx) {
            self.remove(Entity(idx));
        }

        self.entity_to_place.insert(idx, p);
        match self.place_to_entities.get_mut(&p) {
            Some(v) => { v.push(Entity(idx)); return; }
            _ => ()
        };
        // Didn't return above, that means this location isn't indexed
        // yet and needs a brand new container. (Can't do this in match
        // block because borrows.)
        self.place_to_entities.insert(p, vec![Entity(idx)]);
    }

    /// Insert an entity into space.
    pub fn insert_at(&mut self, e: Entity, loc: Location) {
        self.insert(e, At(loc));
    }

    /// Return whether the parent entity or an entity contained in the parent
    /// entity contains entity e.
    pub fn _contains(&self, parent: Entity, Entity(idx): Entity) -> bool {
        match self.entity_to_place.get(&idx) {
            Some(&In(p)) if p == parent => true,
            Some(&In(p)) => self._contains(parent, p),
            _ => false
        }
    }


    /// Insert an entity into container.
    pub fn _insert_in(&mut self, e: Entity, parent: Entity) {
        assert!(!self._contains(e, parent), "Trying to create circular containment");
        self.insert(e, In(parent));
    }

    /// Remove an entity from the space. Other entities that were in the
    /// entity to be removed will be added in the place the entity occupied.
    pub fn remove(&mut self, Entity(idx): Entity) {
        if !self.entity_to_place.contains_key(&idx) { return; }

        let &p = self.entity_to_place.get(&idx).unwrap();

        // Pop out the contents.
        for &content in self.entities_in(Entity(idx)).iter() {
            self.insert(content, p);
        }

        self.entity_to_place.remove(&idx);
        {
            let v = self.place_to_entities.get_mut(&p).unwrap();
            assert!(v.len() > 0);
            if v.len() > 1 {
                // More than one entity present, remove this one, keep the
                // rest.
                let v_idx = v.as_slice().position_elem(&Entity(idx)).unwrap();
                v.swap_remove(v_idx);
                return;
            } else {
                // This was the only entity in the location.
                // Drop the entry for this location from the index.
                // (Need to drop out of scope for borrows reasons)
                assert!((*v)[0] == Entity(idx));
            }
        }
        // We only end up here if we need to clear the container for the
        // location.
        self.place_to_entities.remove(&p);
    }

    fn entities(&self, p: Place) -> Vec<Entity> {
        match self.place_to_entities.get(&p) {
            None => vec![],
            Some(v) => v.clone(),
        }
    }

    /// List entities at a location.
    pub fn entities_at(&self, loc: Location) -> Vec<Entity> {
        self.entities(At(loc))
    }

    /// List entities in a container.
    pub fn entities_in(&self, parent: Entity) -> Vec<Entity> {
        self.entities(In(parent))
    }

    /// Return the place of an entity if the entity is present in the space.
    pub fn get(&self, Entity(idx): Entity) -> Option<Place> {
        self.entity_to_place.get(&idx).map(|&loc| loc)
    }

    /// Flatten to an easily serializable vector.
    fn dump(&self) -> Vec<Elt> {
        let mut ret = vec![];
        for (idx, &loc) in self.entity_to_place.iter() {
            ret.push(Elt(Entity(idx), loc));
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
struct Elt(Entity, Place);

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
