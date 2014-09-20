use std::num::{next_power_of_two};
use image::{GenericImage, SubImage, ImageBuf, Rgba, Pixel};
use util;

pub struct AtlasBuilder {
    images: Vec<ImageBuf<Rgba<u8>>>,
    draw_offsets: Vec<[u32, ..2]>,
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

        let (pos, dim) = util::crop_alpha(&image);
        let cropped = SubImage::new(&mut image, pos[0], pos[1], dim[0], dim[1]);

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
    pub bounds: Vec<([u32, ..2], [u32, ..2])>,
    pub draw_offsets: Vec<[u32, ..2]>,
}

impl Atlas {
    pub fn new(builder: &AtlasBuilder) -> Atlas {
        // Add 1 pixel edges to images to prevent texturing artifacts from
        // adjacent pixels in separate subimages.
        let dims : Vec<[u32, ..2]> = builder.images.iter()
            .map(|img| { let (w, h) = img.dimensions(); [w + 1, h + 1] })
            .collect();

        // Guesstimate the size for the atlas container.
        let total_area = dims.iter().map(|dim| dim[0] * dim[1]).fold(0, |a, b| a + b);
        let mut d = next_power_of_two((total_area as f64).sqrt() as uint) as u32;
        let mut offsets;

        loop {
            assert!(d < 1000000000); // Sanity check
            match util::pack_rectangles([d, d], &dims) {
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

        // Construct subimage rectangles.
        let bounds: Vec<([u32, ..2], [u32, ..2])> = offsets.iter().enumerate()
            .map(|(i, &offset)| (offset, dims[i]))
            .collect();

        let mut draw_offsets = vec![];
        for &i in builder.draw_offsets.iter() {
            draw_offsets.push(i);
        }

        Atlas {
            image: image,
            bounds: bounds,
            draw_offsets: draw_offsets,
        }
    }
}
