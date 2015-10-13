use calx_ecs::Entity;
use spatial::Place;
use self::Ability::*;

/// Ability describes some way of affecting the game world. It is generally
/// attached to a mob or an item.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Ability {
    Multi(Vec<Ability>),
    /// Damage a target for a given amount.
    Damage(i32),
    /// Heal a target for a given amount.
    Heal(i32),
    /// Heals target and self-destructs if target has wounds.
    HealInstant(i32),
}

impl Ability {
    pub fn apply(&self, agent: Option<Entity>, target: Place) {
        if let &Multi(ref abls) = self {
            for abl in abls.iter() {
                abl.apply(agent, target);
            }
            return;
        }

        unimplemented!();
        /*

        // Target entity.
        let te = match target {
            Place::In(e, _) => Some(e),
            Place::At(loc) => loc.main_entity()
        };

        match (self, te) {
            (&Damage(n), Some(e)) => { e.damage(n) }
            (&Heal(n), Some(e))  => { e.heal(n) }
            (&HealInstant(n), Some(e)) => {
                if e.is_wounded() {
                    e.heal(n);
                    if let Some(a) = agent { a.delete() }
                }
            }
            _ => ()
        }
        */
    }
}
