use crate::{
    grammar::{GrammarPart, Noun, Pronoun},
    World,
};
use calx_ecs::Entity;
use serde_derive::{Deserialize, Serialize};

/// The visual representation for an entity
///
/// How this is interpreted depends on the frontend module.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum Icon {
    Player,
    Snake,
    Dreg,
    Ogre,
    Wraith,
    Octopus,
    Bug,
    Ooze,
    Efreet,
    Serpent,
    Bear,
    Spider,
    TotemGuardian,
    Bat,
    Centaur,
    ArmorConstruct,
    CurseSkull,
    GelatinousCube,
    Crocodile,
    Wisp,
    Vortex,
    Moloch,
    Lizardman,
    Centipede,
    FloatingEye,
    EyeHorror,
    Dog,
    Cat,
    Rat,

    PlaceholderMob,
    InvisibleMob,

    Sword,
    Helmet,
    Armor,
    Wand1,
    Wand2,
    Scroll1,
}

/// Entity name and appearance.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Desc {
    pub singular_name: String,
    pub plural_name: Option<String>,
    pub icon: Icon,
}

impl Desc {
    /// Create new description component.
    ///
    /// Give the name with a bar, "goose|geese" to specify an irregular plural.
    /// Regular pluralization always appends 's' to the name or 'es' to names ending with 's', 'x'
    /// or 'z'.
    pub fn new(name: &str, icon: Icon) -> Desc {
        // XXX: Not idiomatic to set this to be called with a non-owned
        // &str instead of a String, I just want to get away from typing
        // .to_string() everywhere with the calls that mostly use string
        // literals.

        let singular_name;
        let plural_name;
        if name.contains('|') {
            let parts: Vec<&str> = name.split('|').collect();
            if parts.len() != 2 {
                panic!("Malformed name string '{}'", name);
            }
            singular_name = parts[0].to_string();
            plural_name = Some(parts[1].to_string());
        } else {
            singular_name = name.to_string();
            plural_name = None;
        }

        Desc {
            singular_name,
            plural_name,
            icon,
        }
    }

    pub fn plural_name(&self) -> String {
        if let Some(plural) = &self.plural_name {
            plural.clone()
        } else if self.singular_name.ends_with('s')
            || self.singular_name.ends_with('x')
            || self.singular_name.ends_with('z')
        {
            format!("{}es", self.singular_name)
        } else {
            format!("{}s", self.singular_name)
        }
    }
}

impl World {
    /// Return visual brush for an entity.
    pub fn entity_icon(&self, e: Entity) -> Option<Icon> { self.ecs().desc.get(e).map(|x| x.icon) }

    pub fn entity_name(&self, e: Entity) -> String {
        if let Some(desc) = self.ecs().desc.get(e) {
            let count = self.count(e);

            if count > 1 {
                format!("{} {}", count, desc.plural_name())
            } else {
                desc.singular_name.clone()
            }
        } else {
            "N/A".to_string()
        }
    }

    pub fn noun(&self, e: Entity) -> Noun {
        let mut ret = Noun::new(self.entity_name(e));
        if self.is_player(e) {
            ret = ret.you().pronoun(Pronoun::They);
        }
        if self.count(e) > 1 {
            ret = ret.plural();
        }
        // TODO: Human mobs get he/she pronoun instead of it.
        ret
    }

    /// Convenience method for formatted messages.
    pub fn subject(&self, e: Entity) -> GrammarPart { GrammarPart::Subject(self.noun(e)) }

    /// Convenience method for formatted messages.
    pub fn object(&self, e: Entity) -> GrammarPart { GrammarPart::Object(self.noun(e)) }

    /// Return the name that can be used to spawn this entity.
    pub fn spawn_name(&self, e: Entity) -> Option<&str> {
        // TODO: Create a special component for this.
        self.ecs()
            .desc
            .get(e)
            .and_then(|desc| Some(&desc.singular_name[..]))
    }
}
