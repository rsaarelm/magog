use crate::tilesheet;
use crate::{Atlas, CharData, FontData, ImageBuffer, ImageData};
use euclid::{rect, size2, vec2, Rect};
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
    image_sheets: HashMap<T, ImageBuffer>,
    atlas_images: HashMap<SubImageSpec<T>, ImageData>,
    atlases: Vec<Atlas>,
    atlas_size: u32,
    next_index: usize,
}

impl<T: Eq + Hash + Clone + Debug> AtlasCache<T> {
    pub fn new(atlas_size: u32, starting_index: usize) -> AtlasCache<T> {
        let mut ret = AtlasCache {
            image_sheets: HashMap::new(),
            atlas_images: HashMap::new(),
            atlases: Vec::new(),
            atlas_size: atlas_size,
            next_index: starting_index,
        };

        ret.new_atlas();

        ret
    }

    pub fn atlases_mut(&mut self) -> slice::IterMut<'_, Atlas> { self.atlases.iter_mut() }

    pub fn atlas_size(&self) -> u32 { self.atlas_size }

    /// Get a drawable `ImageData` corresponding to a subimage specification.
    ///
    /// The named sheet in the `SubImageSpec` key must have been added to the atlas cache before
    /// calling `get`.
    ///
    /// If the added image is larger than `atlas_size` of the atlas cache in either x or y
    /// dimension, the image cannot be added and the method will panic.
    pub fn get<'a>(&'a mut self, key: &SubImageSpec<T>) -> &'a ImageData {
        // This subimage already exists, return it.
        if self.atlas_images.contains_key(key) {
            return &self.atlas_images[key];
        }

        // Subimage does not exist yet, construct the ImageData for it.
        let sub_image = if let Some(image) = self.image_sheets.get(&key.id) {
            // Add a new image to the atlas.

            // First create the buffer image.
            ImageBuffer::from_fn(key.bounds.size.width, key.bounds.size.height, |x, y| {
                image.get_pixel(x + key.bounds.origin.x, y + key.bounds.origin.y)
            })
        } else {
            panic!("Image sheet {:?} not found in cache", key.id);
        };

        assert!(!self.atlases.is_empty());
        let atlas_id = self.atlases.len() - 1;

        if let Some(image_data) = self.atlases[atlas_id].add(&sub_image) {
            // Try to fit it into the current atlas sheet.
            self.atlas_images.insert(key.clone(), image_data);
            &self.atlas_images[key]
        } else if self.atlases[atlas_id].is_empty() {
            // The atlas is empty but adding the image still failed. Assuming image is too large to
            // fit on an atlas sheet even on its own.
            panic!("Image {:?} too large, won't fit in empty atlas.", key);
        } else {
            // Add an empty atlas sheet and retry.
            self.new_atlas();
            assert!(self.atlases[self.atlases.len() - 1].is_empty());
            self.get(key)
        }
    }

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
    pub fn add_sheet<U, I>(&mut self, id: U, sheet: I) -> SubImageSpec<T>
    where
        U: Into<T>,
        I: Into<ImageBuffer>,
    {
        let id = id.into();
        let sheet = sheet.into();

        let ret = SubImageSpec {
            id: id.clone(),
            bounds: rect(0, 0, sheet.size.width, sheet.size.height),
        };

        self.image_sheets.insert(id, sheet);

        ret
    }

    /// Add a named tile source and generate tile data using image properties.
    ///
    /// Calls the `tilesheet_bounds` function to determine tilesheet subimages.
    pub fn add_tilesheet<U, I>(&mut self, id: U, sheet: I) -> Vec<SubImageSpec<T>>
    where
        U: Into<T>,
        I: Into<ImageBuffer>,
    {
        let id = id.into();
        let sheet = sheet.into();

        let ret = tilesheet::tilesheet_bounds(&sheet)
            .into_iter()
            .map(|r| SubImageSpec {
                bounds: r,
                id: id.clone(),
            }).collect();

        self.add_sheet(id, sheet);
        ret
    }

    pub fn add_tilesheet_font<U, I, J>(&mut self, id: U, sheet: I, span: J) -> FontData
    where
        U: Into<T>,
        I: Into<ImageBuffer>,
        J: IntoIterator<Item = char>,
    {
        let tiles = self.add_tilesheet(id, sheet);

        let glyphs = tiles
            .into_iter()
            .map(|i| CharData {
                image: self.get(&i).clone(),
                draw_offset: vec2(0.0, 0.0),
                advance: i.bounds.size.width as f32,
            }).collect::<Vec<_>>();

        let height = glyphs[0].image.size.height as f32;
        let chars = span.into_iter().zip(glyphs.into_iter()).collect();

        FontData { chars, height }
    }
}
