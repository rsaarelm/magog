use crate::spec::EntitySpawn;
use crate::terrain::Terrain;
use calx::{CellVector, FromPrefab, IntoPrefab};
use serde_derive::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt;

pub type Prefab = HashMap<CellVector, (Terrain, Vec<EntitySpawn>)>;

#[derive(Debug, Serialize, Deserialize)]
pub struct MapSave {
    pub map: String,
    pub legend: BTreeMap<char, (Terrain, Vec<EntitySpawn>)>,
}

/// Convert a standard map prefab into an ASCII map with a legend.
///
/// The legend characters are assigned procedurally. The function will fail if the prefab is too
/// complex and the legend generator runs out of separate characters to use.
pub fn build_textmap(
    prefab: &Prefab,
) -> Result<
    (
        HashMap<CellVector, char>,
        BTreeMap<char, (Terrain, Vec<EntitySpawn>)>,
    ),
    Box<dyn Error>,
> {
    const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            αβγδεζηθικλμξπρστφχψω\
                            ΓΔΛΞΠΣΦΨΩ\
                            БГҐДЂЃЄЖЗЙЛЉЊПЎФЦЧЏШЩЪЭЮЯ\
                            àèòùáêõýþâìúãíäîåæçéóëïðñôûöøüÿ\
                            ÀÈÒÙÁÊÕÝÞÂÌÚÃÉÓÄÍÅÆÇËÎÔÏÐÑÖØÛßÜ";

    let chars_f = move |x: &(Terrain, Vec<EntitySpawn>)| {
        let &(ref t, ref e) = x;
        if e.is_empty() {
            t.preferred_map_chars()
        } else {
            ""
        }
    };

    let mut legend_builder = calx::LegendBuilder::new(ALPHABET.to_string(), chars_f);

    let prefab: HashMap<CellVector, _> = prefab
        .clone()
        .into_iter()
        .map(|(p, e)| (p, legend_builder.add(&e)))
        .collect();
    // Values are still results, we need to check if legend building failed.
    if legend_builder.out_of_alphabet {
        return Err("Unable to build legend, scene too complex?".into());
    }

    // Mustn't have non-errs in the build prefab unless out_of_alphabet was flipped.
    let prefab: HashMap<CellVector, _> = prefab.into_iter().map(|(p, e)| (p, e.unwrap())).collect();

    Ok((prefab, legend_builder.legend))
}

impl MapSave {
    pub fn from_prefab(prefab: &Prefab) -> Result<MapSave, Box<dyn Error>> {
        let (prefab, legend) = build_textmap(prefab)?;
        Ok(MapSave::new(prefab, legend))
    }

    pub fn into_prefab(self) -> Result<Prefab, Box<dyn Error>> {
        let (map, legend) = (self.map, self.legend);
        for c in map.chars() {
            if c.is_whitespace() {
                continue;
            }
            if !legend.contains_key(&c) {
                return Err(format!("Unknown map character '{}'", c).into());
            }
        }

        let prefab: HashMap<CellVector, char> = IntoPrefab::into_prefab(map)?;
        let ret: Prefab = prefab
            .into_iter()
            .map(|(p, item)| (p, legend[&item].clone()))
            .collect();
        Ok(ret)
    }

    pub fn new(
        text_prefab: impl IntoIterator<Item = (CellVector, char)>,
        legend: impl IntoIterator<Item = (char, (Terrain, Vec<EntitySpawn>))>,
    ) -> MapSave {
        MapSave {
            map: String::from_prefab(&text_prefab.into_iter().collect()),
            legend: legend.into_iter().collect(),
        }
    }
}

impl fmt::Display for MapSave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Custom RON prettyprint that prints prettier than ron::ser::pretty
        writeln!(f, "(\n    map: \"")?;
        for line in self.map.lines() {
            writeln!(f, "{}", line.trim_end())?;
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
