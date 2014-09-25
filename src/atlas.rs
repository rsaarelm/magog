use std::num::{next_power_of_two};
use image::{GenericImage, SubImage, ImageBuf, Rgba, Pixel};
use util;
use geom::{V2, Rect};

pub struct AtlasBuilder {
    images: Vec<ImageBuf<Rgba<u8>>>,
    draw_offsets: Vec<V2<int>>,
}

impl AtlasBuilder {
    pub fn new() -> AtlasBuilder {
        AtlasBuilder {
            images: vec![],
            draw_offsets: vec![],
        }
    }

    pub fn push<P: Pixel<u8>, I: GenericImage<P>>(
        &mut self, mut image: I) -> uint {

        let Rect(pos, dim) = util::crop_alpha(&image);
        let cropped = SubImage::new(&mut image,
            pos.0 as u32, pos.1 as u32, dim.0 as u32, dim.1 as u32);

        let (w, h) = cropped.dimensions();
        let img = ImageBuf::from_pixels(
            cropped.pixels().map::<Rgba<u8>>(
                |(_x, _y, p)| p.to_rgba())
            .collect(),
            w, h);
        self.images.push(img);
        self.draw_offsets.push(pos);
        self.images.len()
    }
}

pub struct Atlas {
    pub image: ImageBuf<Rgba<u8>>,
    /// Bounds normalized to unit image dimensions (0.0 to 1.0), for easy use as
    /// texture coordinates.
    pub bounds: Vec<Rect<f32>>,
    pub draw_offsets: Vec<V2<int>>,
}

impl Atlas {
    pub fn new(builder: &AtlasBuilder) -> Atlas {
        // Add 1 pixel edges to images to prevent texturing artifacts from
        // adjacent pixels in separate subimages.
        let dims : Vec<V2<int>> = builder.images.iter()
            .map(|img| { let (w, h) = img.dimensions(); V2(w as int + 1, h as int + 1) })
            .collect();

        // Guesstimate the size for the atlas container.
        let total_area = dims.iter().map(|dim| dim.0 * dim.1).fold(0, |a, b| a + b);
        let mut d = next_power_of_two((total_area as f64).sqrt() as uint) as u32;
        let mut offsets;

        loop {
            assert!(d < 1000000000); // Sanity check
            match util::pack_rectangles(V2(d as int, d as int), &dims) {
                Some(ret) => {
                    offsets = ret;
                    break;
                }
                None => {
                    d = d * 2;
                }
            }
        }

        // Blit subimages to atlas image.
        let mut image: ImageBuf<Rgba<u8>> = ImageBuf::new(d, d);
        for (i, &offset) in offsets.iter().enumerate() {
            util::blit(&builder.images[i], &mut image, offset);
        }

        let image_dim = V2(d, d);

        // Construct subimage rectangles.
        let bounds: Vec<Rect<f32>> = offsets.iter().enumerate()
            .map(|(i, &offset)| Rect(scale_vec(offset, image_dim), scale_vec(dims[i], image_dim)))
            .collect();

        let mut draw_offsets = vec![];
        for &i in builder.draw_offsets.iter() {
            draw_offsets.push(i);
        }

        return Atlas {
            image: image,
            bounds: bounds,
            draw_offsets: draw_offsets,
        };

        fn scale_vec(pixel_vec: V2<int>, image_dim: V2<u32>) -> V2<f32> {
            V2(pixel_vec.0 as f32 / image_dim.0 as f32,
              pixel_vec.1 as f32 / image_dim.1 as f32)
        }
    }
}
