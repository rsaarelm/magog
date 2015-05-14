use std::marker::{PhantomData};
use std::collections::{VecMap};
use std::fmt::{Debug};
use collections::enum_set::{CLike};
use image::{GenericImage, SubImage, Pixel};
use ::geom::{V2};
use super::canvas::{CanvasBuilder, Image};

/// A cache that loads sprites and stores them efficiently indexed by
/// enum constants given by the user.
///
/// Use an enum type with #[repr(usize)] that contains names for all the
/// sprites as the key type.
pub struct SpriteCache<T> {
    cache: VecMap<Image>,
    // Lock the cache into the specific enum type even though it doesn't
    // show up in the actual implementation, just to keep usage clearer.
    phantom: PhantomData<T>,
}

impl<T: CLike+Debug+Copy> SpriteCache<T> {
    pub fn new() -> SpriteCache<T> {
        SpriteCache {
            cache: VecMap::new(),
            phantom: PhantomData,
        }
    }
    
    /// Try to retrieve an image with the given identifier.
    pub fn get(&self, key: T) -> Option<Image> {
        self.cache.get(&key.to_usize()).map(|&x| x)
    }

    /// Add an image to the CanvasBuilder atlas for the given identifier key.
    ///
    /// Trying to add multiple sprites with the same key is considered a
    /// bug and will cause a panic.
    pub fn add<P, I>(&mut self, builder: &mut CanvasBuilder, key: T, offset: V2<i32>, image: &I)
        where P: Pixel<Subpixel=u8> + 'static,
              I: GenericImage<Pixel=P> {
        let idx = key.to_usize();
        assert!(self.cache.get(&idx).is_none(), format!("Adding sprite {:?} twice", key));

        self.cache.insert(idx, builder.add_image(offset, image));
    }

    /// Add multiple images from a sprite sheet.
    ///
    /// The sheet images are assumed to be in a grid with sprite_size
    /// cells. The sheet is read from top to bottom in left-to-right
    /// rows. Sprites are read and given cache keys corresponding to the
    /// values in the keys Vec as long as there are items left in keys.
    /// If there are more keys in the keys Vec than there are sprites in
    /// the sprite sheet, the function will panic.
    pub fn batch_add<P, I>(&mut self, builder: &mut CanvasBuilder, keys: Vec<T>,
                           offset: V2<i32>, sprite_size: V2<u32>, sprite_sheet: &mut I)
        where P: Pixel<Subpixel=u8> + 'static,
              I: GenericImage<Pixel=P> + 'static
    {
        let (w, h) = sprite_sheet.dimensions(); 
        let (cols, rows) = (w / sprite_size.0, h / sprite_size.1);
        assert!(keys.len() as u32 <= rows * cols, "More keys specified than there are sprites on the sheet");
        for i in 0..keys.len() {
            let sub = SubImage::new(sprite_sheet, (i as u32 % cols) * w, (i as u32 / cols) * h, w, h); 
            self.add(builder, keys[i], offset, &sub);
        }
    }
}
