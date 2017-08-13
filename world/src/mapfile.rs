use Prefab;
use calx_grid;
use errors::*;
use form::Form;
use std::collections::BTreeMap;
use std::io;
use terrain::Terrain;
use ron;

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

    let mut legend_builder = calx_grid::LegendBuilder::new(ALPHABET.to_string(), chars_f);

    let prefab = prefab.clone().map(|e| legend_builder.add(&e));
    // Values are still results, we need to check if legend building failed.
    if legend_builder.out_of_alphabet {
        return Err("Unable to build legend, scene too complex?".into());
    }

    // Mustn't have non-errs in the build prefab unless out_of_alphabet was flipped.
    let prefab = prefab.map(|e| e.unwrap());

    // Add a leading newline because the first line of a multiline string literal in the
    // serialization might be indented to a different depth and mess up how the top of the map
    // looks to humans.
    let map = format!("\n{}", prefab.hexmap_display());
    let legend = legend_builder.legend;

    write!(output, "{}", ron::ser::pretty::to_string(&MapSave { map, legend })?)?;
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
    let prefab: calx_grid::Prefab<char> = calx_grid::Prefab::from_text_hexmap(&save.map);
    Ok(prefab.map(|item| { save.legend[&item].clone() }))
}

#[derive(Debug, Serialize, Deserialize)]
struct MapSave {
    pub map: String,
    pub legend: BTreeMap<char, (Terrain, Vec<String>)>,
}
