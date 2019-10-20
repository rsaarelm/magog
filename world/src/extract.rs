use crate::item::Slot;
use crate::mutate::Mutate;
use crate::world::{Loadout, World};
use crate::Query;
use calx_ecs::Entity;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Data of an entity that has been extracted out of the world
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExternalEntity {
    pub loadout: Loadout,
    pub contents: BTreeMap<Slot, ExternalEntity>,
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
    pub(crate) fn _inject(&mut self, external_entity: &ExternalEntity) -> Entity {
        let entity = external_entity.loadout.make(self.ecs_mut());

        for (slot, e) in &external_entity.contents {
            let item = self._inject(e);
            self.spatial.equip(item, entity, *slot);
        }

        self.rebuild_stats(entity);

        entity
    }
}
