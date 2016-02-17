/// Macro for constructing sets of named sprites
///
/// Currently hardcoded to use CYAN / #00FFFF as the transparent color key.
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
            pub fn get(self, idx: usize) -> usize {
                BRUSH_CACHE.with(|c| c.borrow().get(self).expect("Brush not initialized")[idx])
            }

            /// Build the actual sprites in a canvas for the enum set.
            pub fn init<S>(builder: &mut S)
                where S: $crate::ImageStore
            {
                BRUSH_CACHE.with(|c| { *c.borrow_mut() = build_brushes(builder); });
            }
        }

        thread_local!(static BRUSH_CACHE: RefCell<$crate::IndexCache<Brush, Vec<usize>>> = RefCell::new(IndexCache::new()));

        // TODO: Builder as generic ImageStore
        fn build_brushes<S>(builder: &mut S) -> $crate::IndexCache<Brush, Vec<usize>>
            where S: $crate::ImageStore
        {
            use image;
            use $crate::color_key;

            let mut ret = $crate::IndexCache::new();

            fn load(data: &'static [u8]) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
                // Don't force the macro user have calx_color imported, use
                // the convert from u32 interface for colors.
                let cyan = 0x00FFFFFF;
                color_key(&image::load_from_memory(data).unwrap(), cyan)
            }

            $({
                let mut sheet = load(include_bytes!($filename));

                $({
                    let mut frames = Vec::new();

                    $({
                        frames.push(builder.add_image(
                                [$xcenter, $ycenter],
                                &image::SubImage::new(&mut sheet, $x, $y, $xdim, $ydim)));
                    })*

                    ret.insert($enumname::$brushname, frames);
                })*

            })*

            ret
        }
    }
}
