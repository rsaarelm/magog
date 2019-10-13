use crate::atlas::Atlas;
use crate::tilesheet;
use crate::{CharData, FontData, ImageData};
use euclid::default::Rect;
use euclid::{rect, size2, vec2};
use image::{Pixel, RgbaImage};
use log::warn;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::slice;

/// Fetch key for atlas images.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SubImageSpec<T> {
    pub id: T,
    pub bounds: Rect<u32>,
}

impl<T> SubImageSpec<T> {
    pub fn new<U: Into<T>>(id: U, x: u32, y: u32, width: u32, height: u32) -> SubImageSpec<T> {
        SubImageSpec {
            id: id.into(),
            bounds: rect(x, y, width, height),
        }
    }
}

/// Updateable incremental cache for multiple texture atlases.
pub struct AtlasCache<T> {
    image_sheets: HashMap<T, RgbaImage>,
    atlas_images: HashMap<SubImageSpec<T>, ImageData>,
    atlases: Vec<Atlas>,
    atlas_size: u32,
    next_index: usize,
    solid: ImageData,
}

impl<T: Eq + Hash + Clone + Debug> Default for AtlasCache<T> {
    fn default() -> AtlasCache<T> {
        let atlas_size = 1024;

        let mut atlas0 = Atlas::new(0, size2(atlas_size, atlas_size));

        // Hacky hack: Add one-pixel pure-white texture for drawing solid shapes without swapping
        // out the atlas texture. Assume atlas behavior will make it go in a predictable position
        // so we can just generate the image.
        let solid = RgbaImage::from_pixel(1, 1, Pixel::from_channels(0xff, 0xff, 0xff, 0xff));
        let solid = atlas0.add(&solid).unwrap();

        // Let's just make it so we can use a dirty hack and assume it shows up in origin without
        // even having AtlasCache reference available...
        assert!(solid.tex_coords.origin == euclid::point2(0.0, 0.0));

        AtlasCache {
            image_sheets: HashMap::new(),
            atlas_images: HashMap::new(),
            atlases: vec![atlas0],
            atlas_size,
            next_index: 1,
            solid,
        }
    }
}

impl<T: Eq + Hash + Clone + Debug> AtlasCache<T> {
    pub fn atlases_mut(&mut self) -> slice::IterMut<'_, Atlas> { self.atlases.iter_mut() }

    pub fn atlas_size(&self) -> u32 { self.atlas_size }

    /// Get a drawable `ImageData` corresponding to a subimage specification.
    ///
    /// The named sheet in the `SubImageSpec` key must have been added to the atlas cache before
    /// calling `get`.
    ///
    /// If the added image is larger than `atlas_size` of the atlas cache in either x or y
    /// dimension, the image cannot be added and the method will panic.
    pub fn get<'a>(&'a mut self, key: &SubImageSpec<T>) -> Option<ImageData> {
        // This subimage already exists, return it.
        if self.atlas_images.contains_key(key) {
            return self.atlas_images.get(key).cloned();
        }

        // Subimage does not exist yet, construct the ImageData for it.
        let sub_image = if let Some(image) = self.image_sheets.get(&key.id) {
            // Add a new image to the atlas.

            // First create the buffer image.
            RgbaImage::from_fn(key.bounds.size.width, key.bounds.size.height, |x, y| {
                *image.get_pixel(x + key.bounds.origin.x, y + key.bounds.origin.y)
            })
        } else {
            warn!("Image sheet {:?} not found in cache", key.id);
            return None;
        };

        assert!(!self.atlases.is_empty());
        let atlas_id = self.atlases.len() - 1;

        if let Some(image_data) = self.atlases[atlas_id].add(&sub_image) {
            // Try to fit it into the current atlas sheet.
            self.atlas_images.insert(key.clone(), image_data);
            self.atlas_images.get(key).cloned()
        } else if self.atlases[atlas_id].is_empty() {
            // The atlas is empty but adding the image still failed. Assuming image is too large to
            // fit on an atlas sheet even on its own.
            warn!("Image {:?} too large, won't fit in empty atlas.", key);
            None
        } else {
            // Add an empty atlas sheet and retry.
            self.new_atlas();
            assert!(self.atlases[self.atlases.len() - 1].is_empty());
            self.get(key)
        }
    }

    pub fn get_solid(&self) -> &ImageData { &self.solid }

    fn new_atlas(&mut self) {
        self.atlases.push(Atlas::new(
            self.next_index,
            size2(self.atlas_size, self.atlas_size),
        ));
        self.next_index += 1;
    }

    /// Add a named tile source sheet into the cache.
    ///
    /// Return the `SubImageSpec` for the entire sheet in case it is a single image that should be
    /// used as is.
    pub fn add_sheet(&mut self, id: impl Into<T>, sheet: impl Into<RgbaImage>) -> SubImageSpec<T> {
        let sheet = sheet.into();
        let id = id.into();

        let ret = SubImageSpec {
            id: id.clone(),
            bounds: rect(0, 0, sheet.width(), sheet.height()),
        };

        self.image_sheets.insert(id, sheet);

        ret
    }

    /// Add a named tile source and generate tile data using image properties.
    ///
    /// Calls the `tilesheet_bounds` function to determine tilesheet subimages.
    pub fn add_tilesheet(
        &mut self,
        id: impl Into<T>,
        sheet: impl Into<RgbaImage>,
    ) -> Vec<SubImageSpec<T>> {
        let sheet = sheet.into();
        let id = id.into();

        let ret = tilesheet::tilesheet_bounds(&sheet)
            .into_iter()
            .map(|r| SubImageSpec {
                bounds: r,
                id: id.clone(),
            })
            .collect();

        self.add_sheet(id, sheet);
        ret
    }

    pub fn add_tilesheet_font(
        &mut self,
        id: impl Into<T>,
        sheet: impl Into<RgbaImage>,
        span: impl IntoIterator<Item = char>,
    ) -> FontData {
        let tiles = self.add_tilesheet(id, sheet);

        let glyphs = tiles
            .into_iter()
            .map(|i| CharData {
                image: self.get(&i).unwrap(),
                draw_offset: vec2(0, 0),
                advance: i.bounds.size.width as i32,
            })
            .collect::<Vec<_>>();

        let height = glyphs[0].image.size.height as i32;
        let chars = span.into_iter().zip(glyphs.into_iter()).collect();

        FontData { chars, height }
    }
}
