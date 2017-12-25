use calx::templatize;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Pronoun {
    He,
    She,
    It,
    They,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Noun {
    is_you: bool,
    name: String,
    pronoun: Pronoun,
}

impl Noun {
    pub fn new(name: String) -> Noun {
        Noun {
            is_you: false,
            pronoun: Pronoun::It,
            name,
        }
    }

    pub fn pronoun(mut self, pronoun: Pronoun) -> Noun {
        self.pronoun = pronoun;
        self
    }

    pub fn you(mut self) -> Noun {
        self.is_you = true;
        self
    }

    pub fn is_proper_noun(&self) -> bool { is_capitalized(&self.name) }

    pub fn the_name(&self) -> String {
        if self.is_you {
            "you".to_string()
        } else if self.is_proper_noun() {
            self.name.to_string()
        } else {
            format!("the {}", self.name)
        }
    }

    pub fn a_name(&self) -> String {
        if self.is_you {
            "you".to_string()
        } else if self.is_proper_noun() {
            self.name.to_string()
        } else {
            // TODO: Add look-up table of irregular words ('honor', 'unit') as they show up in game
            // text.
            let article = if is_vowel(self.name.chars().next().unwrap_or('\0')) {
                "an"
            } else {
                "a"
            };
            format!("{} {}", article, self.name)
        }
    }

    pub fn they(&self) -> String {
        if self.is_you {
            "you".to_string()
        } else {
            match self.pronoun {
                Pronoun::He => "he".to_string(),
                Pronoun::She => "she".to_string(),
                Pronoun::It => "it".to_string(),
                Pronoun::They => "they".to_string(),
            }
        }
    }

    pub fn them(&self) -> String {
        if self.is_you {
            "you".to_string()
        } else {
            match self.pronoun {
                Pronoun::He => "him".to_string(),
                Pronoun::She => "her".to_string(),
                Pronoun::It => "it".to_string(),
                Pronoun::They => "them".to_string(),
            }
        }
    }

    pub fn possessive(&self) -> String {
        if self.is_you {
            "your".to_string()
        } else if self.is_proper_noun() {
            format!("{}'s", self.name.to_string())
        } else {
            format!("the {}'s", self.name)
        }
    }

    pub fn reflexive(&self) -> String {
        if self.is_you {
            "yourself".to_string()
        } else {
            match self.pronoun {
                Pronoun::He => "himself".to_string(),
                Pronoun::She => "herself".to_string(),
                Pronoun::It => "itself".to_string(),
                Pronoun::They => "themselves".to_string(),
            }
        }
    }
}

pub trait Templater {
    /// Clear any state changes that happened during convert sequence.
    fn reset(&mut self) {}

    /// Convert a token.
    fn convert(&mut self, token: &str) -> Option<String> {
        let _ = token;
        None
    }

    fn format(&mut self, text: &str) -> Result<String, String> {
        let ret = templatize(
            |token| {
                if let Some(mut s) = self.convert(&token.to_lowercase()) {
                    if is_capitalized(token) {
                        s = capitalize(&s);
                    }
                    Ok(s)
                } else {
                    Err(token.to_string())
                }
            },
            text,
        )?;

        Ok(ret)
    }
}

pub struct EmptyTemplater;

impl Templater for EmptyTemplater {}

pub struct SubjectTemplater {
    subject: Noun,
    used_pronoun_they: bool,
}

impl SubjectTemplater {
    pub fn new(subject: Noun) -> SubjectTemplater {
        SubjectTemplater {
            subject,
            used_pronoun_they: false,
        }
    }
}

impl Templater for SubjectTemplater {
    fn reset(&mut self) { self.used_pronoun_they = false; }

    fn convert(&mut self, token: &str) -> Option<String> {
        let ret = match token {
            "one" => self.subject.the_name(),
            "one's" => self.subject.possessive(),
            "oneself" => self.subject.reflexive(),
            "they" => {
                if !self.subject.is_you && self.subject.pronoun == Pronoun::They {
                    // If the pronoun for the subject actually is 'they', the grammar must change
                    // from "He does" to "They do".
                    //
                    // Since verb follows subject in English sentences, we can fortunately set the
                    // flag here to accomplish this.
                    self.used_pronoun_they = true;
                }
                self.subject.they()
            }

            // Second / third person verb endings and irregular verbs.
            // All of these are assummed to apply to subject.
            // hit/hits
            "s" => {
                if self.subject.is_you || self.used_pronoun_they {
                    "".to_string()
                } else {
                    "s".to_string()
                }
            }
            // slash/slashes
            "es" => {
                if self.subject.is_you || self.used_pronoun_they {
                    "".to_string()
                } else {
                    "es".to_string()
                }
            }
            // parry/parries
            "ies" => {
                if self.subject.is_you || self.used_pronoun_they {
                    "y".to_string()
                } else {
                    "ies".to_string()
                }
            }
            "is" | "are" => {
                if self.subject.is_you || self.used_pronoun_they {
                    "are".to_string()
                } else {
                    "is".to_string()
                }
            }
            "has" | "have" => {
                if self.subject.is_you || self.used_pronoun_they {
                    "have".to_string()
                } else {
                    "has".to_string()
                }
            }

            _ => {
                return None;
            }
        };
        Some(ret)
    }
}

pub struct ObjectTemplater {
    parent: SubjectTemplater,
    object: Noun,
}

impl ObjectTemplater {
    pub fn new(parent: SubjectTemplater, object: Noun) -> ObjectTemplater {
        ObjectTemplater { parent, object }
    }
}

impl Templater for ObjectTemplater {
    fn reset(&mut self) { self.parent.reset(); }

    fn convert(&mut self, token: &str) -> Option<String> {
        let ret = match token {
            "another" => self.object.the_name(),
            "another's" => self.object.possessive(),
            "them" => self.object.them(),

            _ => {
                return self.parent.convert(token);
            }
        };
        Some(ret)
    }
}

pub fn is_capitalized(word: &str) -> bool {
    word.chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
}

pub fn capitalize(word: &str) -> String {
    let mut iter = word.chars();
    let cap = if let Some(c) = iter.next() {
        c.to_uppercase()
    } else {
        return String::new();
    };
    cap.chain(iter).collect()
}

pub fn is_vowel(c: char) -> bool {
    // If accented chars are used, they need to be added here...
    match c.to_lowercase().next().unwrap_or('\0') {
        'a' | 'e' | 'i' | 'o' | 'u' => true,
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::{Noun, ObjectTemplater, Pronoun, SubjectTemplater, Templater};

    #[test]
    fn test_caps_helpers() {
        use super::{capitalize, is_capitalized};
        for &(text, cap) in &[
            ("", ""),
            ("a", "A"),
            ("A", "A"),
            ("Abc", "Abc"),
            ("abc", "Abc"),
            ("aBC", "ABC"),
            ("ABC", "ABC"),
            ("æ", "Æ"),
            ("æìë", "Æìë"),
        ] {
            assert_eq!(&capitalize(text), cap);
            assert_eq!(
                is_capitalized(text),
                !text.is_empty() && &capitalize(text) == text
            );
        }
    }

    fn make_noun(name: &str) -> Noun {
        let ret = Noun::new(name.to_string());

        match name {
            "PLAYER" => ret.you(),
            "Alexander" => ret.pronoun(Pronoun::He),
            "Athena" => ret.pronoun(Pronoun::She),
            "Tiresias" => ret.pronoun(Pronoun::They),
            _ => ret,
        }
    }

    fn parse_subj<'a>(test_script: &'a str) -> Vec<(&'a str, &'a str, &'a str)> {
        test_script
            .lines()
            .filter_map(|s| {
                let s = s.trim();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            })
            .collect::<Vec<&str>>()
            .chunks(3)
            .map(|a| (a[0], a[1], a[2]))
            .collect()
    }

    fn parse_obj<'a>(test_script: &'a str) -> Vec<(&'a str, &'a str, &'a str, &'a str)> {
        test_script
            .lines()
            .filter_map(|s| {
                let s = s.trim();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            })
            .collect::<Vec<&str>>()
            .chunks(4)
            .map(|a| (a[0], a[1], a[2], a[3]))
            .collect()
    }

    #[test]
    fn test_templating_1() {
        for (subject, template, message) in parse_subj(
            "PLAYER
                 [One] drink[s] the potion.
                 You drink the potion.

                 goblin
                 [One] drink[s] the potion.
                 The goblin drinks the potion.

                 PLAYER
                 [One] rush[es] through the door.
                 You rush through the door.

                 goblin
                 [One] rush[es] through the door.
                 The goblin rushes through the door.

                 PLAYER
                 The spear runs [one] through.
                 The spear runs you through.

                 goblin
                 The spear runs [one] through.
                 The spear runs the goblin through.

                 Alexander
                 The spear runs [one] through.
                 The spear runs Alexander through.

                 PLAYER
                 [One] [is] the chosen one. [They] [have] a rock.
                 You are the chosen one. You have a rock.

                 Athena
                 [One] [is] the chosen one. [They] [have] a rock.
                 Athena is the chosen one. She has a rock.

                 Tiresias
                 [One] [has] a rock. [They] [are] the chosen one.
                 Tiresias has a rock. They are the chosen one.

                 PLAYER
                 [One] nimbly parr[ies] the blow.
                 You nimbly parry the blow.

                 goblin
                 [One] nimbly parr[ies] the blow.
                 The goblin nimbly parries the blow.",
        ).into_iter()
        {
            let mut t = SubjectTemplater::new(make_noun(subject));
            assert_eq!(t.format(template), Ok(message.to_string()));
        }
    }

    #[test]
    fn test_templating_2() {
        for (subject, object, template, message) in parse_obj(
            "PLAYER
                 goblin
                 [One] hit[s] [another].
                 You hit the goblin.

                 goblin
                 PLAYER
                 [One] hit[s] [another].
                 The goblin hits you.

                 PLAYER
                 goblin
                 [One] chase[s] after [them].
                 You chase after it.

                 PLAYER
                 wand of death
                 [One] zap[s] [oneself] with [another].
                 You zap yourself with the wand of death.

                 Alexander
                 wand of speed
                 [One] zap[s] [oneself] with [another].
                 Alexander zaps himself with the wand of speed.

                 PLAYER
                 Alexander
                 [One] chase[s] after [them].
                 You chase after him.
                 ",
        ).into_iter()
        {
            let mut t =
                ObjectTemplater::new(SubjectTemplater::new(make_noun(subject)), make_noun(object));
            assert_eq!(t.format(template), Ok(message.to_string()));
        }
    }
}
