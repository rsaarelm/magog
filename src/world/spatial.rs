use std::collections::{VecMap};
use bst::{TreeMap};
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use super::entity::{Entity};
use super::location::{Location};
use super::item::Slot;
use self::Place::*;

/// Entities can be placed either on open locations or inside other entities.
/// A sum type will represent this nicely.
#[derive(Copy, Eq, PartialEq, Clone, PartialOrd, Ord, Debug, RustcEncodable, RustcDecodable)]
pub enum Place {
    At(Location),
    In(Entity, Option<Slot>),
}

/// Spatial index for game entities
pub struct Spatial {
    place_to_entities: TreeMap<Place, Vec<Entity>>,
    entity_to_place: VecMap<Place>,
}

impl Spatial {
    pub fn new() -> Spatial {
        Spatial {
            place_to_entities: TreeMap::new(),
            entity_to_place: VecMap::new(),
        }
    }

    fn insert(&mut self, Entity(idx): Entity, p: Place) {
        // Remove the entity from its old position.
        self.single_remove(Entity(idx));

        if let In(_, Some(_)) = p {
            // Slotted in-places are a special case that can hold at most one entity.
            if self.place_to_entities.contains_key(&p) {
                panic!("Equipping to an occupied inventory slot");
            }
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
    pub fn contains(&self, parent: Entity, Entity(idx): Entity) -> bool {
        match self.entity_to_place.get(&idx) {
            Some(&In(p, _)) if p == parent => true,
            Some(&In(p, _)) => self.contains(parent, p),
            _ => false
        }
    }

    /// Insert an entity into container.
    pub fn insert_in(&mut self, e: Entity, parent: Entity) {
        assert!(!self.contains(e, parent), "Trying to create circular containment");
        self.insert(e, In(parent, None));
    }

    /// Insert an entity into an equipment slot. Will panic if there already
    /// is an item present in the slot.
    pub fn equip(&mut self, e: Entity, parent: Entity, slot: Slot) {
        self.insert(e, In(parent, Some(slot)));
    }

    /// Remove an entity from the local structures but do not pop out its
    /// items. Unless the entity is added back in or the contents are handled
    /// somehow, this will leave the spatial index in an inconsistent state.
    fn single_remove(&mut self, Entity(idx): Entity) {
        if !self.entity_to_place.contains_key(&idx) { return; }

        let &p = self.entity_to_place.get(&idx).unwrap();
        self.entity_to_place.remove(&idx);

        {
            let v = self.place_to_entities.get_mut(&p).unwrap();
            assert!(v.len() > 0);
            if v.len() > 1 {
                // More than one entity present, remove this one, keep the
                // rest.
                let v_idx = (&v[..]).position_elem(&Entity(idx)).unwrap();
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

    /// Remove an entity from the space. Entities contained in the entity will
    /// also be removed from the space.
    pub fn remove(&mut self, Entity(idx): Entity) {
        // Remove the contents
        for &content in self.entities_in(Entity(idx)).iter() {
            self.remove(content);
        }
        self.single_remove(Entity(idx));
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
        // XXX: Can't make the API return an iterator (more efficient than
        // running collect) since the chain depends on a closure that captures
        // the 'parent' parameter from the outside scope, and closures can't
        // be typed in the return signature.
        self.place_to_entities.lower_bound(&In(parent, None))
            // Consume the contingent elements for the parent container.
            .take_while(|&(ref k, _)| if let &&In(ref p, _) = k { *p == parent } else { false })
            // Flatten the Vec results into a single stream.
            .flat_map(|(_, ref v)| v.iter())
            .map(|&x| x)
            .collect()
    }

    pub fn entity_equipped(&self, parent: Entity, slot: Slot) -> Option<Entity> {
        match self.place_to_entities.get(&In(parent, Some(slot))) {
            None => None,
            Some(v) => {
                assert!(v.len() == 1, "Slot entity container corrupt");
                Some(v[0])
            }
        }
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

#[derive(Clone, RustcDecodable, RustcEncodable)]
struct Elt(Entity, Place);

impl Decodable for Spatial {
    fn decode<D: Decoder>(d: &mut D) -> Result<Spatial, D::Error> {
        Ok(Spatial::slurp(try!(Decodable::decode(d))))
    }
}

impl Encodable for Spatial {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        self.dump().encode(s)
    }
}

#[cfg(test)]
mod test {
    use super::{Place};
    use world::item::Slot;
    use world::entity::Entity;

    #[test]
    fn test_place_adjacency() {
        // Test that the Place type gets a lexical ordering where elements in
        // the same parent entity get sorted next to each other, and that None
        // is the minimum value for the slot option.
        //
        // This needs to be right for the containment logic to function, but
        // it's not obvious which way the derived lexical order sorts, so put
        // an unit test here to check it out.
        let mut places = vec![
            Place::In(Entity(0), Some(Slot::Melee)),
            Place::In(Entity(1), None),
            Place::In(Entity(0), Some(Slot::Ranged)),
            Place::In(Entity(0), None),
        ];

        places.sort();
        assert_eq!(places,
            vec![
                Place::In(Entity(0), None),
                Place::In(Entity(0), Some(Slot::Melee)),
                Place::In(Entity(0), Some(Slot::Ranged)),
                Place::In(Entity(1), None),
            ]);
    }
}
