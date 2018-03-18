use {Atlas, ImageBuffer, ImageData};
use euclid::{rect, Rect, size2};
use std::collections::HashMap;
use std::slice;
use tilesheet;

/// Fetch key for atlas images.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SubImageSpec {
    pub sheet_name: String,
    pub bounds: Rect<u32>,
}

impl SubImageSpec {
    pub fn new(name: &str, x: u32, y: u32, width: u32, height: u32) -> SubImageSpec {
        SubImageSpec {
            sheet_name: name.to_string(),
            bounds: rect(x, y, width, height),
        }
    }
}

/// Updateable incremental cache for multiple texture atlases.
///
/// This will set image type to `usize`, the backend implementer is resposible for mapping
/// the consecutive integers to actual backend atlas texture values.
pub struct AtlasCache {
    image_sheets: HashMap<String, ImageBuffer>,
    atlas_images: HashMap<SubImageSpec, ImageData<usize>>,
    atlases: Vec<Atlas<usize>>,
    atlas_size: u32,
    next_index: usize,
}

impl AtlasCache {
    pub fn new(atlas_size: u32, starting_index: usize) -> AtlasCache {
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

    pub fn atlases_mut(&mut self) -> slice::IterMut<Atlas<usize>> { self.atlases.iter_mut() }

    pub fn atlas_size(&self) -> u32 { self.atlas_size }

    /// Get a drawable `ImageData` corresponding to a subimage specification.
    ///
    /// The named sheet in the `SubImageSpec` key must have been added to the atlas cache before
    /// calling `get`.
    ///
    /// If the added image is larger than `atlas_size` of the atlas cache in either x or y
    /// dimension, the image cannot be added and the method will panic.
    pub fn get<'a>(&'a mut self, key: &SubImageSpec) -> &'a ImageData<usize> {
        // This subimage already exists, return it.
        if self.atlas_images.contains_key(key) {
            return &self.atlas_images[key];
        }

        // Subimage does not exist yet, construct the ImageData for it.
        let sub_image = if let Some(image) = self.image_sheets.get(&key.sheet_name) {
            // Add a new image to the atlas.

            // First create the buffer image.
            ImageBuffer::from_fn(key.bounds.size.width, key.bounds.size.height, |x, y| {
                image.get_pixel(x + key.bounds.origin.x, y + key.bounds.origin.y)
            })
        } else {
            panic!("Image sheet {} not found in cache", key.sheet_name);
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
    pub fn add_sheet<S, I>(&mut self, name: S, sheet: I)
    where
        S: Into<String>,
        I: Into<ImageBuffer>,
    {
        self.image_sheets.insert(name.into(), sheet.into());
    }

    /// Add a named tile source and generate tile data using image properties.
    ///
    /// Calls the `tilesheet_bounds` function to determine tilesheet subimages.
    pub fn add_tilesheet<S, I>(&mut self, name: S, sheet: I) -> Vec<SubImageSpec>
    where
        S: Into<String>,
        I: Into<ImageBuffer>,
    {
        let name = name.into();
        let sheet = sheet.into();

        let ret = tilesheet::tilesheet_bounds(&sheet)
            .into_iter()
            .map(|r| SubImageSpec {
                bounds: r,
                sheet_name: name.clone(),
            })
            .collect();

        self.add_sheet(name, sheet);
        ret
    }
}
