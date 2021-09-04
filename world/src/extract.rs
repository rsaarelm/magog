use crate::{spec::EntitySpawn, world::Loadout, Distribution, Rng, Slot, World};
use calx_ecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;

/// Data of an entity that has been extracted out of the world
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct ExternalEntity {
    pub loadout: Loadout,
    pub contents: BTreeMap<Slot, ExternalEntity>,
}

impl ExternalEntity {
    pub fn new(loadout: Loadout) -> ExternalEntity {
        ExternalEntity {
            loadout,
            ..Default::default()
        }
    }

    /// Create an external entity from entity database names.
    pub fn sample_from_name(rng: &mut Rng, name: &str) -> Result<Self, ()> {
        if let Ok(spawn) = EntitySpawn::from_str(name) {
            Ok(spawn.sample(rng))
        } else {
            Err(())
        }
    }

    /// Syntactic sugar, use a fixed RNG for spawns for which random variation doesn't matter.
    pub fn from_name(name: &str) -> Result<Self, ()> {
        Self::sample_from_name(&mut calx::seeded_rng(&1), name)
    }
}

impl World {
    /// Extract an entity and its contents into a standalone structure.
    pub fn extract(&self, e: Entity) -> Option<ExternalEntity> {
        if !self.ecs().contains(e) {
            return None;
        }
        let loadout = Loadout::get(self.ecs(), e);
        let contents = self
            .entities_in(e)
            .into_iter()
            .filter_map(|(slot, e)| {
                if let Some(e) = self.extract(e) {
                    Some((slot, e))
                } else {
                    None
                }
            })
            .collect();
        Some(ExternalEntity { loadout, contents })
    }

    /// Inject a standalone entity structure into the world state.
    pub(crate) fn inject(&mut self, external_entity: &ExternalEntity) -> Entity {
        let entity = external_entity.loadout.make(self.ecs_mut());

        for (slot, e) in &external_entity.contents {
            let item = self.inject(e);
            self.spatial.equip(item, entity, *slot);
        }

        self.rebuild_stats(entity);

        entity
    }
}
