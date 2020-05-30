use crate::spec::EntitySpawn;
use crate::{Location, Sector, Terrain};
use calx::{tiled, CellVector, FromPrefab, IntoPrefab};
use euclid::vec2;
use serde_derive::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

pub type Prefab = HashMap<CellVector, (Terrain, Vec<EntitySpawn>)>;

const LEGEND_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                               abcdefghijklmnopqrstuvwxyz\
                               αβγδεζηθικλμξπρστφχψω\
                               ΓΔΛΞΠΣΦΨΩ\
                               БГҐДЂЃЄЖЗЙЛЉЊПЎФЦЧЏШЩЪЭЮЯ\
                               àèòùáêõýþâìúãíäîåæçéóëïðñôûöøüÿ\
                               ÀÈÒÙÁÊÕÝÞÂÌÚÃÉÓÄÍÅÆÇËÎÔÏÐÑÖØÛßÜ";

const TILED_TILE_WIDTH: f32 = 16.0;
const TILED_TILE_HEIGHT: f32 = 16.0;

/// Types that can be described in pseudo-natural language.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Parseable<T>(pub T);

serde_plain::derive_deserialize_from_str!(Parseable<(Terrain, Vec<EntitySpawn>)>, "parseable");
serde_plain::derive_serialize_from_display!(Parseable<(Terrain, Vec<EntitySpawn>)>);

impl std::str::FromStr for Parseable<(Terrain, Vec<EntitySpawn>)> {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').map(|s| s.trim()).collect();
        if parts.is_empty() {
            Err("No terrain set")?;
        }

        let terrain = Terrain::from_str(parts[0])?;
        let mut spawns = Vec::new();
        for spawn in parts.iter().skip(1) {
            spawns.push(EntitySpawn::from_str(spawn)?);
        }

        Ok(Parseable((terrain, spawns)))
    }
}

impl std::fmt::Display for Parseable<(Terrain, Vec<EntitySpawn>)> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (ref terrain, ref spawns) = self.0;
        write!(f, "{}", terrain.name())?;
        for c in spawns {
            write!(f, ", {}", c)?;
        }
        Ok(())
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct MapPatch {
    pub map: String,
    pub legend: BTreeMap<char, Parseable<(Terrain, Vec<EntitySpawn>)>>,
}

impl MapPatch {
    /// Construct a `MapPatch` from cell data.
    ///
    /// Will fail if there are so many unique cell types that the legend alphabet gets exhausted.
    /// Does not preserve offset of the input, the result will have origin at the top left corner
    /// of the bounding box containing all the map cells.
    pub fn new(
        cells: impl IntoIterator<Item = (CellVector, (Terrain, Vec<EntitySpawn>))>,
    ) -> Result<MapPatch, Box<dyn Error>> {
        let chars_f = move |x: &(Terrain, Vec<EntitySpawn>)| {
            let &(ref t, ref e) = x;
            if e.is_empty() {
                t.preferred_map_chars()
            } else {
                ""
            }
        };

        let mut legend_builder = calx::LegendBuilder::new(LEGEND_ALPHABET.to_string(), chars_f);

        let mut prefab = HashMap::new();
        for (k, v) in cells.into_iter() {
            if let Ok(c) = legend_builder.add(&v) {
                prefab.insert(k, c);
            } else {
                Err("Unable to build legend, scene too complex?")?;
            }
        }

        let map = calx::DenseTextMap::from_prefab(&prefab).0;
        let legend = legend_builder
            .legend
            .into_iter()
            .map(|(c, k)| (c, Parseable(k)))
            .collect();

        Ok(MapPatch { map, legend })
    }

    pub fn iter(&self) -> impl Iterator<Item = (CellVector, (Terrain, Vec<EntitySpawn>))> + '_ {
        calx::DenseTextMap(&self.map)
            .into_prefab::<HashMap<CellVector, char>>()
            .unwrap()
            .into_iter()
            .map(move |(p, c)| (p, (self.legend.get(&c).unwrap().0).clone()))
    }
}

impl fmt::Display for MapPatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", outline::into_outline(self).unwrap())
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct PatchData {
    pub offset: Location,

    #[serde(flatten)]
    pub patch: MapPatch,
}

impl fmt::Display for PatchData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", outline::into_outline(self).unwrap())
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct WorldData {
    pub patches: Vec<PatchData>,
}

impl fmt::Display for WorldData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", outline::into_outline(self).unwrap())
    }
}

impl TryFrom<tiled::Map> for WorldData {
    type Error = Box<dyn Error>;

    fn try_from(tiled: tiled::Map) -> Result<Self, Self::Error> {
        // Find layer with magic name "surface" to fix z level with, otherwise z=0 is top layer and
        // it counts down from there.
        let starting_z = tiled
            .layers
            .iter()
            .filter(|a| a.is_tile_layer())
            .enumerate()
            .find(|(_, a)| a.name().to_lowercase() == "surface")
            .map(|(i, _)| (i as i32))
            .unwrap_or(0);

        let mut terrain_map = HashMap::new();
        let mut spawn_map = HashMap::new();

        // Process Tiled layers
        let mut z = starting_z;
        for layer in tiled.layers.iter().rev() {
            let loc = Location::new(0, 0, z as i16);

            if let Some(i) = layer.iter_tiles() {
                for (pos, t) in i {
                    let loc = loc + vec2(pos.x, pos.y);
                    if let Some(t) = tiled_to_terrain(t) {
                        terrain_map.insert(loc, t);
                    }
                }

                z -= 1;
            }

            if let Some(i) = layer.iter_objects() {
                for o in i {
                    // XXX: Tiled makes Y be off by one, has upwards-pointing Y-axis?
                    let loc = loc
                        + vec2(
                            (o.x / TILED_TILE_WIDTH).round() as i32,
                            (o.y / TILED_TILE_HEIGHT).round() as i32 - 1,
                        );
                    spawn_map.insert(loc, tiled_to_spawn(o)?);
                }
            }
        }

        // Construct intermediate storage layers.
        struct Layer {
            // Extents of the terrain data, used to compute offset for the segment.
            min_x: i32,
            min_y: i32,
            cells: HashMap<CellVector, (Terrain, Vec<EntitySpawn>)>,
        }

        impl Default for Layer {
            fn default() -> Self {
                Layer {
                    min_x: std::i32::MAX,
                    min_y: std::i32::MAX,
                    cells: Default::default(),
                }
            }
        }

        let mut layers = BTreeMap::new();
        for (loc, c) in terrain_map {
            let sector = Sector::from(loc);
            let pos = Location::from(sector).v2_at(loc).unwrap();

            let layer = layers.entry(sector).or_insert(Layer::default());
            layer.min_x = layer.min_x.min(pos.x);
            layer.min_y = layer.min_y.min(pos.y);
            layer.cells.insert(pos, (c, Vec::new() as Vec<EntitySpawn>));
        }

        for (loc, c) in spawn_map {
            let sector = Sector::from(loc);
            let pos = Location::from(sector).v2_at(loc).unwrap();

            let layer = layers.entry(sector).or_insert(Layer::default());
            if let Some(cell) = layer.cells.get_mut(&pos) {
                cell.1.push(c);
            } else {
                Err(format!("Object spawn {}: {:?} outside terrain", c, loc))?;
            }
        }

        // Construct WorldData instance from intermediate data.

        let mut patches = Vec::new();
        for (sector, layer) in layers.into_iter() {
            let offset = Location::from(sector) + vec2(layer.min_x, layer.min_y);
            let patch = MapPatch::new(layer.cells.into_iter())?;
            patches.push(PatchData { offset, patch });
        }

        Ok(WorldData { patches })
    }
}

fn tiled_to_terrain(tiled_id: u32) -> Option<Terrain> {
    /// Hardcoded tileset used in Tiled maps. Edit as needed.
    const TILED_TILES: [Terrain; 17] = [
        // FIXME: Only have valid terrains in the list, keep this simple...
        Terrain::Empty,
        Terrain::Empty,
        Terrain::Empty,
        Terrain::Ground,
        Terrain::Water,
        Terrain::Empty, // TODO: Monolith terrain
        Terrain::Tree,
        Terrain::Wall,
        Terrain::Rock,
        Terrain::Window,
        Terrain::Door,
        Terrain::Downstairs,
        Terrain::Upstairs,
        Terrain::Grass,
        Terrain::Shallows,
        Terrain::Sand,
        Terrain::Empty, // TODO: Mountain face terrain
    ];
    if let Some(&t) = TILED_TILES.get(tiled_id as usize) {
        if t != Terrain::Empty {
            Some(t)
        } else {
            None
        }
    } else {
        None
    }
}

fn tiled_to_spawn(object: &tiled::Object) -> Result<EntitySpawn, Box<dyn Error>> {
    const TILED_SPAWNS_OFFSET: usize = 130;
    const TILED_SPAWNS: [&str; 4] = ["player", "dreg", "ooze", "sword"];

    if !object.name.is_empty() {
        return Ok(EntitySpawn::from_str(&object.name)?);
    }

    let gid = object.gid as usize;

    if gid >= TILED_SPAWNS_OFFSET {
        if let Some(s) = TILED_SPAWNS.get(gid - TILED_SPAWNS_OFFSET) {
            return Ok(EntitySpawn::from_str(s)?);
        }
    }
    Err("Bad spawn")?
}

#[cfg(test)]
mod test {
    #[test]
    fn test_legend_parseable() {
        use super::Parseable;
        use crate::spec::EntitySpawn;
        use crate::Terrain;
        use std::str::FromStr;

        type LegendEntry = Parseable<(Terrain, Vec<EntitySpawn>)>;

        assert!(LegendEntry::from_str("").is_err());

        assert_eq!(
            LegendEntry::from_str("grass").unwrap(),
            Parseable((Terrain::Grass, Vec::new()))
        );
        assert_eq!(
            LegendEntry::from_str("grass, sword").unwrap(),
            Parseable((
                Terrain::Grass,
                vec![EntitySpawn::from_str("sword").unwrap()]
            ))
        );
        assert_eq!(
            LegendEntry::from_str("sand, scroll of lightning, snake").unwrap(),
            Parseable((
                Terrain::Sand,
                vec![
                    EntitySpawn::from_str("scroll of lightning").unwrap(),
                    EntitySpawn::from_str("snake").unwrap()
                ]
            ))
        );
        assert_eq!(
            format!(
                "{}",
                Parseable((
                    Terrain::Sand,
                    vec![
                        EntitySpawn::from_str("scroll of lightning").unwrap(),
                        EntitySpawn::from_str("snake").unwrap()
                    ]
                ))
            ),
            "sand, scroll of lightning, snake"
        );
    }
}
