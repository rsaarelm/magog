use std::fmt;
use std::io;
use std::str::FromStr;
use std::collections::BTreeMap;
use rustc_serialize::Decodable;
use toml;
use calx_grid;
use Prefab;
use terrain::Terrain;
use form::Form;
use errors::*;

pub fn save_prefab<W: io::Write>(output: &mut W, prefab: &Prefab) -> Result<()> {
    const ALPHABET: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
             abcdefghijklmnopqrstuvwxyz\
             0123456789";

    let chars_f = move |x: &(Terrain, Vec<String>)| {
        let &(ref t, ref e) = x;
        if e.is_empty() { t.preferred_map_chars() } else { "" }
    };

    let mut legend_builder = calx_grid::LegendBuilder::new(ALPHABET.to_string(), chars_f);

    let prefab = prefab.clone().map(|e| legend_builder.add(&e));
    // Values are still results, we need to check if legend building failed.
    if legend_builder.out_of_alphabet {
        return Err("Unable to build legend, scene too complex?".into());
    }

    // Mustn't have non-errs in the build prefab unless out_of_alphabet was flipped.
    let prefab = prefab.map(|e| e.unwrap());

    let legend = legend_builder.legend
                               .into_iter()
                               .map(|(c, t)| {
                                   (c,
                                    LegendItem {
                                       t: format!("{:?}", t.0),
                                       e: t.1.clone(),
                                   })
                               })
                               .collect::<BTreeMap<char, LegendItem>>();

    let save = MapSave {
        map: format!("{}", prefab.hexmap_display()),
        legend: legend,
    };

    write!(output, "{}", save)?;
    Ok(())
}

pub fn load_prefab<I: io::Read>(input: &mut I) -> Result<Prefab> {
    let mut s = String::new();
    input.read_to_string(&mut s)?;
    let mut parser = toml::Parser::new(&s);
    let mut decoder = toml::Decoder::new(
        toml::Value::Table(parser.parse().ok_or_else(|| format!("Parse errors: {:?}", parser.errors))?));
    let save = MapSave::decode(&mut decoder)?;

    // Validate the prefab
    for i in save.legend.values() {
        if Terrain::from_str(&i.t).is_err() {
            return Err(format!("Unknown terrain type '{}'", i.t).into());
        }

        for e in &i.e {
            if Form::named(&e).is_none() {
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
    Ok(prefab.map(|item| {
        let e = save.legend.get(&item).unwrap();
        let terrain = Terrain::from_str(&e.t).unwrap();
        let spawns = e.e.clone();
        (terrain, spawns)
    }))
}

/// Type for maps saved into disk.
#[derive(Debug, RustcEncodable, RustcDecodable)]
struct MapSave {
    pub map: String,
    pub legend: BTreeMap<char, LegendItem>,
}

impl fmt::Display for MapSave {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TOML formatted output.
        writeln!(f, "map = '''")?;
        for line in self.map.lines() {
            writeln!(f, "{}", line.trim_right())?;
        }
        writeln!(f, "'''\n")?;
        writeln!(f, "[legend]")?;
        for (k, v) in self.legend.iter() {
            writeln!(f, "\"{}\" = {}", k, v)?;
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, RustcEncodable, RustcDecodable)]
struct LegendItem {
    /// Terrain
    pub t: String,
    /// Entities
    pub e: Vec<String>,
}

impl fmt::Display for LegendItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TOML formatted output.
        write!(f, "{{ t = \"{}\", e = [", self.t)?;
        self.e.iter().next().map_or(Ok(()), |e| write!(f, "\"{}\"", e))?;
        for e in self.e.iter().skip(1) {
            write!(f, ", \"{}\"", e)?;
        }
        write!(f, "] }}")
    }
}
