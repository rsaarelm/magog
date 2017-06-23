use euclid::{Rect, rect, size2};
use image::{self, GenericImage};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::slice;
use tilesheet;
use vitral::{self, ImageBuffer};
use vitral_atlas::Atlas;

/// Fetch key for atlas images.
#[derive(Clone, PartialEq, Eq, Debug)]
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

impl Hash for SubImageSpec {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sheet_name.hash(state);
        self.bounds.origin.hash(state);
        self.bounds.size.hash(state);
    }
}

pub struct AtlasCache {
    image_sheets: HashMap<String, ImageBuffer>,
    atlas_images: HashMap<SubImageSpec, vitral::ImageData<usize>>,
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

    pub fn get<'a>(&'a mut self, key: &SubImageSpec) -> &'a vitral::ImageData<usize> {
        if self.atlas_images.contains_key(key) {
            &self.atlas_images[key]
        } else {
            self.add(key)
        }
    }

    pub fn atlases_mut(&mut self) -> slice::IterMut<Atlas<usize>> { self.atlases.iter_mut() }

    fn add<'a>(&'a mut self, key: &SubImageSpec) -> &'a vitral::ImageData<usize> {
        assert!(!self.atlas_images.contains_key(key));

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
            self.atlas_images.insert(key.clone(), image_data);
            &self.atlas_images[key]
        } else if self.atlases[atlas_id].is_empty() {
            panic!("Image {:?} too large, won't fit in empty atlas.", key);
        } else {
            // Try adding a new atlas.
            self.new_atlas();
            assert!(self.atlases[self.atlases.len() - 1].is_empty());
            self.add(key)
        }
    }

    fn new_atlas(&mut self) {
        self.atlases
            .push(Atlas::new(self.next_index,
                             size2(self.atlas_size, self.atlas_size)));
        self.next_index += 1;
    }

    pub fn add_sheet(&mut self, name: String, sheet: ImageBuffer) {
        self.image_sheets.insert(name, sheet);
    }

    /// Load PNG from bytes to use in image specs.
    pub fn load_png(&mut self, name: String, data: &[u8]) -> Result<(), image::ImageError> {
        let img = image::load(Cursor::new(data), image::ImageFormat::PNG)?;
        self.add_sheet(name, convert_image(&img));
        Ok(())
    }

    /// Load PNG and use the tile detection algorithm to generate tilesheet image specs.
    pub fn load_tilesheet(
        &mut self,
        name: String,
        data: &[u8],
    ) -> Result<Vec<SubImageSpec>, image::ImageError> {
        let img = image::load(Cursor::new(data), image::ImageFormat::PNG)?;
        let ret = tilesheet::bounds(&img)
            .into_iter()
            .map(|r| {
                     SubImageSpec {
                         bounds: r,
                         sheet_name: name.clone(),
                     }
                 })
            .collect();
        self.add_sheet(name, convert_image(&img));
        Ok(ret)
    }
}

fn convert_image<G>(input: &G) -> ImageBuffer
    where G: GenericImage<Pixel = image::Rgba<u8>>
{
    let (w, h) = input.dimensions();

    let build_fn = |x, y| {
        let rgba = input.get_pixel(x, y).data;

        (rgba[3] as u32) << 24 | (rgba[2] as u32) << 16 | (rgba[1] as u32) << 8 | (rgba[0] as u32)
    };
    ImageBuffer::from_fn(w, h, build_fn)
}
