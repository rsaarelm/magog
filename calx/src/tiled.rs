//! Data structure for Tiled map editor JSON format.

use euclid::default::Point2D;
use euclid::point2;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::{self, FromIterator, IntoIterator};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Orientation {
    #[serde(rename = "orthogonal")]
    Orthogonal,
    #[serde(rename = "isometric")]
    Isometric,
    #[serde(rename = "staggered")]
    Staggered,
    #[serde(rename = "hexagonal")]
    Hexagonal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub data: Vec<u32>,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChunkMap(pub Vec<Chunk>);

impl FromIterator<(Point2D<i32>, u32)> for ChunkMap {
    fn from_iter<I: IntoIterator<Item = (Point2D<i32>, u32)>>(iter: I) -> Self {
        const CHUNK_WIDTH: i32 = 16;
        const CHUNK_HEIGHT: i32 = 16;
        let mut chunks = HashMap::new();

        // Place all points into corresponding chunks.
        for (pt, c) in iter.into_iter() {
            // Get the top corner of the chunk where this point goes in.
            let x = if pt.x >= 0 {
                pt.x / CHUNK_WIDTH
            } else {
                ((pt.x + 1) / CHUNK_WIDTH) - 1
            };
            let y = if pt.y >= 0 {
                pt.y / CHUNK_HEIGHT
            } else {
                ((pt.y + 1) / CHUNK_HEIGHT) - 1
            };

            let origin = point2(x * CHUNK_WIDTH, y * CHUNK_HEIGHT);
            let offset = (pt.x - origin.x + (pt.y - origin.y) * CHUNK_WIDTH) as usize;
            chunks
                .entry(origin)
                .or_insert_with(|| vec![0; (CHUNK_WIDTH * CHUNK_HEIGHT) as usize])[offset] = c;
        }

        // Turn chunks into ChunkMap data.
        ChunkMap(
            chunks
                .into_iter()
                .map(|(orig, data): (Point2D<_>, Vec<_>)| Chunk {
                    data,
                    width: CHUNK_WIDTH as u32,
                    height: CHUNK_HEIGHT as u32,
                    x: orig.x,
                    y: orig.y,
                })
                .collect(),
        )
    }
}

impl ChunkMap {
    /// Iterate the nonzero points in a chunk map
    pub fn iter(&self) -> impl Iterator<Item = (Point2D<i32>, u32)> + '_ {
        let mut i = 0;
        let mut p = 0;
        iter::from_fn(move || {
            if i < self.0.len() && p >= self.0[i].data.len() {
                i += 1;
                p = 0;
            }
            if i >= self.0.len() {
                return None;
            }

            let chunk = &self.0[i];
            let pitch = chunk.width as i32;
            let offset = p as i32;
            let ret = (
                point2(chunk.x + offset % pitch, chunk.y + offset / pitch),
                chunk.data[p],
            );
            p += 1;
            Some(ret)
        })
        .filter_map(|(p, c)| if c == 0 { None } else { Some((p, c)) })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Layer {
    #[serde(rename = "tilelayer")]
    TileLayer {
        name: String,
        id: u32,
        visible: bool,
        opacity: f32,

        x: i32,
        y: i32,
        width: u32,
        height: u32,

        #[serde(skip_serializing_if = "Option::is_none")]
        chunks: Option<ChunkMap>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<Vec<u32>>,
        // Not supported: compression, encoding
    },
    #[serde(rename = "objectgroup")]
    ObjectGroup {
        name: String,
        id: u32,
        visible: bool,
        opacity: f32,

        x: i32,
        y: i32,

        draworder: String,
        objects: Vec<Object>,
    },
    // Not supported: imagelayer, group
}

impl Layer {
    pub fn name(&self) -> &str {
        match self {
            Layer::TileLayer { name, .. } => name,
            Layer::ObjectGroup { name, .. } => name,
        }
    }

    pub fn is_tile_layer(&self) -> bool {
        matches!(self, Layer::TileLayer { .. })
    }

    pub fn iter_tiles(&self) -> Option<Box<dyn Iterator<Item = (Point2D<i32>, u32)> + '_>> {
        if let Layer::TileLayer {
            x,
            y,
            width,
            chunks,
            ref data,
            ..
        } = self
        {
            if let Some(chunks) = chunks {
                Some(Box::new(chunks.iter()))
            } else {
                let data = data
                    .as_ref()
                    .expect("Invalid map, has neither chunks nor data");
                let width = *width as usize;

                Some(Box::new(data.iter().enumerate().filter_map(
                    move |(i, c)| {
                        if *c == 0 {
                            return None;
                        }

                        let p = point2(x + (i % width) as i32, y + (i / width) as i32);

                        Some((p, *c))
                    },
                )))
            }
        } else {
            None
        }
    }

    pub fn iter_objects(&self) -> Option<impl Iterator<Item = &Object>> {
        if let Layer::ObjectGroup { objects, .. } = self {
            Some(objects.iter())
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Object {
    #[serde(rename = "type")]
    pub type_: String,
    pub gid: u32,
    pub id: u32,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tileset {
    pub columns: u32,
    pub tilecount: u32,
    pub tileheight: u32,
    pub tilewidth: u32,
    pub spacing: u32,
    pub firstgid: u32,
    pub image: PathBuf,
    pub imageheight: u32,
    pub imagewidth: u32,
    pub margin: u32,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MapProperty {
    #[serde(rename = "boolean")]
    Boolean { name: String, value: bool },
    #[serde(rename = "color")]
    Color {
        name: String,
        value: String, // TODO: Deserialize to colory type from "#rrggbbaa"
    },
    #[serde(rename = "file")]
    File { name: String, value: PathBuf },
    #[serde(rename = "float")]
    Float { name: String, value: f64 },
    #[serde(rename = "int")]
    Int { name: String, value: i32 },
    #[serde(rename = "string")]
    String { name: String, value: String },
}

/// Toplevel Tiled map data type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Map {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backgroundcolor: Option<String>,
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>,
    pub infinite: bool,
    pub nextlayerid: u32,
    pub nextobjectid: u32,
    pub orientation: Orientation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<MapProperty>>,
    pub renderorder: String,
    pub tiledversion: String,
    pub tileheight: u32,
    pub tilewidth: u32,
    pub version: f32,
    pub tilesets: Vec<Tileset>,
}
