//! Macro for constructing sets of named sprites

#[macro_export]
macro_rules! brush {
    {
        $enumname:ident {
            // Image file to load brushes from.
            $([$filename:expr,
              // Named brushes
              $([$brushname:ident,
                  // Frames for the named brush as rectangles in the current
                  // image file.
                  $($xcenter:expr, $ycenter:expr, $xdim:expr, $ydim:expr, $x:expr, $y:expr),+
              ])+
            ])+
        }
    } =>
    {
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
        pub enum $enumname {
            $(
                $($brushname,)*
            )*
        }

        cache_key!($enumname);

        impl $enumname {
            /// Get the sprite image.
            pub fn get(self, idx: usize) -> Image {
                BRUSH_CACHE.with(|c| c.borrow().get(self).expect("Brush not initialized")[idx])
            }

            /// Build the actual sprites in a canvas for the enum set.
            pub fn init(builder: &mut CanvasBuilder) {
                BRUSH_CACHE.with(|c| { *c.borrow_mut() = build_brushes(builder); });
            }
        }

        thread_local!(static BRUSH_CACHE: RefCell<$crate::IndexCache<Brush, Vec<Image>>> = RefCell::new(IndexCache::new()));

        fn build_brushes(builder: &mut $crate::backend::CanvasBuilder) -> $crate::IndexCache<Brush, Vec<Image>> {
            use image;
            use $crate::{V2, color_key, color};

            let mut ret = $crate::IndexCache::new();

            fn load(data: &'static [u8]) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
                // XXX: Shouldn't hardcode the color::CYAN part, factor out
                // the loader?
                color_key(&image::load_from_memory(data).unwrap(), color::CYAN)
            }

            $({
                let mut sheet = load(include_bytes!($filename));

                $({
                    let mut frames = Vec::new();

                    $({
                        // Offset vector is the negative of the center
                        // position. Center position is nicer for humans doing
                        // data entry.
                        frames.push(builder.add_image(
                                V2(-$xcenter, -$ycenter),
                                &image::SubImage::new(&mut sheet, $x, $y, $xdim, $ydim)));
                    })*

                    ret.insert($enumname::$brushname, frames);
                })*

            })*

            ret
        }
    }
}
