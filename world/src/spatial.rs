use self::Place::*;
use crate::{Location, Slot, World};
use calx_ecs::Entity;
use serde;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::slice;

impl World {
    /// Return the location of an entity.
    ///
    /// Returns the location of the containing entity for entities inside
    /// containers. It is possible for entities to not have a location.
    pub fn location(&self, e: Entity) -> Option<Location> {
        match self.spatial.get(e) {
            Some(Place::At(loc)) => Some(loc),
            Some(Place::In(container, _)) => self.location(container),
            _ => None,
        }
    }

    /// Return all entities in the world.
    pub fn entities(&self) -> slice::Iter<'_, Entity> { self.ecs.iter() }

    // XXX: Would be nicer if entities_at returned an iterator. Probably want to wait for impl
    // Trait return types before jumping to this.

    /// Return entities at the given location.
    pub fn entities_at(&self, loc: Location) -> Vec<Entity> {
        self.spatial.entities_at(loc)
    }

    /// Return entities inside another entity.
    pub fn entities_in(&self, parent: Entity) -> Vec<(Slot, Entity)> {
        self.spatial.entities_in(parent)
    }

    /// Return true if entity contains nothing.
    pub fn is_empty(&self, e: Entity) -> bool { self.spatial.is_empty(e) }

    /// Return the item parent has equipped in slot.
    pub fn entity_equipped(
        &self,
        parent: Entity,
        slot: Slot,
    ) -> Option<Entity> {
        self.spatial.entity_equipped(parent, slot)
    }

    pub fn entity_contains(&self, parent: Entity, child: Entity) -> bool {
        self.spatial.contains(parent, child)
    }

    /// Return slot entity is equipped in.
    pub fn entity_slot(&self, e: Entity) -> Option<Slot> {
        if let Some(Place::In(_, slot)) = self.spatial.get(e) {
            Some(slot)
        } else {
            None
        }
    }

    pub(crate) fn set_entity_location(&mut self, e: Entity, loc: Location) {
        self.spatial.insert_at(e, loc);
    }
}

/// Entities can be placed either on open locations or inside other entities.
/// A sum type will represent this nicely.
#[derive(
    Copy, Eq, PartialEq, Clone, PartialOrd, Ord, Debug, Serialize, Deserialize,
)]
pub enum Place {
    At(Location),
    // XXX: Implementation can store multiple entities in a single Slot-place, but in practice
    // it's always one entity per slot, the player's inventory needs to be able to have gaps in it
    // because of the UI. Using separate indices for Location -> Vec<Entity> and (Entity, Slot) ->
    // Entity might be worth considering to simplify this.
    In(Entity, Slot),
}

/// Spatial index for game entities
pub struct Spatial {
    place_to_entities: BTreeMap<Place, Vec<Entity>>,
    entity_to_place: BTreeMap<Entity, Place>,
}

impl Spatial {
    pub fn new() -> Spatial {
        Spatial {
            place_to_entities: BTreeMap::new(),
            entity_to_place: BTreeMap::new(),
        }
    }

    /// The most general insert method.
    pub fn insert(&mut self, e: Entity, p: Place) {
        // Remove the entity from its old position.
        self.single_remove(e);

        if let In(parent, _) = p {
            assert!(
                !self.contains(e, parent),
                "Trying to create circular containment"
            );

            assert!(
                !self.place_to_entities.contains_key(&p),
                "Equipping to an occupied inventory slot"
            );
        }

        self.entity_to_place.insert(e, p);
        if let Some(v) = self.place_to_entities.get_mut(&p) {
            v.push(e);
            return;
        }
        // Didn't return above, that means this location isn't indexed
        // yet and needs a brand new container. (Can't do this in match
        // block because borrows.)
        self.place_to_entities.insert(p, vec![e]);
    }

    /// Insert an entity into space.
    pub fn insert_at(&mut self, e: Entity, loc: Location) {
        self.insert(e, At(loc));
    }

    /// Return whether the parent entity or an entity contained in the parent
    /// entity contains entity e.
    pub fn contains(&self, parent: Entity, e: Entity) -> bool {
        match self.entity_to_place.get(&e) {
            Some(&In(p, _)) if p == parent => true,
            Some(&In(p, _)) => self.contains(parent, p),
            _ => false,
        }
    }

    /// Insert an entity into an equipment slot. Will panic if there already
    /// is an item present in the slot.
    pub fn equip(&mut self, e: Entity, parent: Entity, slot: Slot) {
        assert!(
            !self.contains(e, parent),
            "Trying to create circular containment"
        );
        self.insert(e, In(parent, slot));
    }

    /// Remove an entity from the local structures but do not pop out its
    /// items. Unless the entity is added back in or the contents are handled
    /// somehow, this will leave the spatial index in an inconsistent state.
    fn single_remove(&mut self, e: Entity) {
        if !self.entity_to_place.contains_key(&e) {
            return;
        }

        let &p = &self.entity_to_place[&e];
        self.entity_to_place.remove(&e);

        {
            let v = self.place_to_entities.get_mut(&p).unwrap();
            assert!(!v.is_empty());
            if v.len() > 1 {
                // More than one entity present, remove this one, keep the
                // rest.
                for i in 0..v.len() {
                    if v[i] == e {
                        v.swap_remove(i);
                        return;
                    }
                }
                panic!("Entity being removed from place it's not in");
            } else {
                // This was the only entity in the location.
                // Drop the entry for this location from the index.
                // (Need to drop out of scope for borrows reasons)
                assert_eq!((*v)[0], e);
            }
        }
        // We only end up here if we need to clear the container for the
        // location.
        self.place_to_entities.remove(&p);
    }

    /// Remove an entity from the space. Entities contained in the entity will
    /// also be removed from the space.
    pub fn remove(&mut self, e: Entity) {
        // Remove the contents
        for (_, content) in &self.entities_in(e) {
            self.remove(*content);
        }
        self.single_remove(e);
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
    pub fn entities_in(&self, parent: Entity) -> Vec<(Slot, Entity)> {
        self.place_to_entities
            .range(In(parent, Slot::Bag(0))..)
            // Consume the contiguous elements for the parent container.
            // This expects the ordering of the `Place` type to group contents
            // of the same parent together.
            .take_while(|&(k, _)| {
                if let In(ref p, _) = *k {
                    *p == parent
                } else {
                    false
                }
            })
            .flat_map(|(p, e)| {
                if let In(_, slot) = p {
                    e.iter().map(move |e| (*slot, *e))
                } else {
                    panic!("Corrupt place_to_entities spatial index");
                }
            })
            .collect()
    }

    pub fn is_empty(&self, entity: Entity) -> bool {
        if let Some((In(ref parent, _), _)) = self
            .place_to_entities
            .range(In(entity, Slot::Bag(0))..)
            .next()
        {
            *parent != entity
        } else {
            true
        }
    }

    pub fn entity_equipped(
        &self,
        parent: Entity,
        slot: Slot,
    ) -> Option<Entity> {
        match self.place_to_entities.get(&In(parent, slot)) {
            None => None,
            Some(v) => {
                assert_eq!(v.len(), 1, "Slot entity container corrupt");
                Some(v[0])
            }
        }
    }

    /// Return the place of an entity if the entity is present in the space.
    pub fn get(&self, e: Entity) -> Option<Place> {
        self.entity_to_place.get(&e).cloned()
    }

    /// Flatten to an easily serializable vector.
    fn dump(&self) -> Vec<Elt> {
        let mut ret = vec![];
        for (&e, &loc) in &self.entity_to_place {
            ret.push(Elt(e, loc));
        }
        ret
    }

    /// Construct from the serialized vector.
    fn slurp(dump: Vec<Elt>) -> Spatial {
        let mut ret = Spatial::new();

        for &Elt(e, loc) in &dump {
            ret.insert(e, loc);
        }
        ret
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Elt(Entity, Place);

impl serde::Serialize for Spatial {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.dump().serialize(s)
    }
}

impl<'a> serde::Deserialize<'a> for Spatial {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        Ok(Spatial::slurp(serde::Deserialize::deserialize(d)?))
    }
}

#[cfg(test)]
mod test {
    use super::{Place, Spatial};
    use crate::item::Slot;
    use crate::location::Location;
    use crate::world::Ecs;

    #[test]
    fn test_place_adjacency() {
        let mut ecs = Ecs::new();
        let e1 = ecs.make();
        let e2 = ecs.make();

        // Test that the Place type gets a lexical ordering where elements in the same parent
        // entity get sorted next to each other, and that Bag(0) is the minimum value for the slot
        // option.
        //
        // This needs to be right for the containment logic to function, but it's not obvious which
        // way the derived lexical order sorts, so put an unit test here to check it out.
        let mut places = vec![
            Place::In(e1, Slot::RightHand),
            Place::In(e2, Slot::Bag(0)),
            Place::In(e1, Slot::Ranged),
            Place::In(e1, Slot::Bag(1)),
            Place::In(e1, Slot::Bag(0)),
        ];

        places.sort();
        assert_eq!(
            places,
            vec![
                Place::In(e1, Slot::Bag(0)),
                Place::In(e1, Slot::Bag(1)),
                Place::In(e1, Slot::Ranged),
                Place::In(e1, Slot::RightHand),
                Place::In(e2, Slot::Bag(0)),
            ]
        );
    }

    #[test]
    fn test_serialization() {
        use ron::de;
        use ron::ser;

        let mut ecs = Ecs::new();
        let e1 = ecs.make();
        let e2 = ecs.make();

        let mut spatial = Spatial::new();
        let p1 = Place::At(Location::new(10, 10, 0));
        let p2 = Place::In(e1, Slot::Bag(0));
        spatial.insert(e1, p1);
        spatial.insert(e2, p2);

        let saved =
            ser::to_string(&spatial).expect("Spatial serialization failed");
        let spatial2: Spatial =
            de::from_str(&saved).expect("Spatial deserialization failed");

        assert_eq!(spatial2.get(e1), Some(p1));
        assert_eq!(spatial2.get(e2), Some(p2));
    }
}
