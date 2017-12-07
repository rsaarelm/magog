use Prefab;
use calx::{self, IntoPrefab};
use errors::*;
use form::Form;
use ron;
use std::collections::BTreeMap;
use std::fmt;
use std::io;
use terrain::Terrain;

pub fn save_prefab<W: io::Write>(output: &mut W, prefab: &Prefab) -> Result<()> {
    const ALPHABET: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                    abcdefghijklmnopqrstuvwxyz\
                                    0123456789";

    let chars_f = move |x: &(Terrain, Vec<String>)| {
        let &(ref t, ref e) = x;
        if e.is_empty() {
            t.preferred_map_chars()
        } else {
            ""
        }
    };

    let mut legend_builder = calx::LegendBuilder::new(ALPHABET.to_string(), chars_f);

    let prefab: calx::Prefab<_> = prefab
        .clone()
        .into_iter()
        .map(|(p, e)| (p, legend_builder.add(&e)))
        .collect();
    // Values are still results, we need to check if legend building failed.
    if legend_builder.out_of_alphabet {
        return Err("Unable to build legend, scene too complex?".into());
    }

    // Mustn't have non-errs in the build prefab unless out_of_alphabet was flipped.
    let prefab: calx::Prefab<_> = prefab.into_iter().map(|(p, e)| (p, e.unwrap())).collect();

    let map = format!("{}", String::from(prefab));
    let legend = legend_builder.legend;

    write!(output, "{}", MapSave { map, legend })?;
    Ok(())
}

pub fn load_prefab<I: io::Read>(input: &mut I) -> Result<Prefab> {
    let mut s = String::new();
    input.read_to_string(&mut s)?;
    let save: MapSave = ron::de::from_str(&s)?;

    // Validate the prefab
    for i in save.legend.values() {
        for e in &i.1 {
            if Form::named(e).is_none() {
                return Err(format!("Unknown entity spawn '{}'", e).into());
            }
        }
    }
    for c in save.map.chars() {
        if c.is_whitespace() {
            continue;
        }
        if !save.legend.contains_key(&c) {
            return Err(format!("Unknown map character '{}'", c).into());
        }
    }

    // Turn map into prefab.
    let (map, legend) = (save.map, save.legend);
    let prefab: calx::Prefab<char> = IntoPrefab::try_into(map)?;
    Ok(prefab
        .into_iter()
        .map(|(p, item)| (p, legend[&item].clone()))
        .collect())
}

#[derive(Debug, Serialize, Deserialize)]
struct MapSave {
    pub map: String,
    pub legend: BTreeMap<char, (Terrain, Vec<String>)>,
}

impl fmt::Display for MapSave {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Custom RON prettyprint that prints prettier than ron::ser::pretty
        writeln!(f, "(\n    map: \"")?;
        for line in self.map.lines() {
            writeln!(f, "{}", line.trim_right())?;
        }
        writeln!(f, "\",\n")?;
        writeln!(f, "    legend: {{")?;
        for (k, v) in &self.legend {
            writeln!(f, "        {:?}: ({:?}, {:?}),", k, v.0, v.1)?;
        }
        writeln!(f, "    }}")?;
        writeln!(f, ")")
    }
}
