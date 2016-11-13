use image::GenericImage;
use euclid::{Point2D, Rect, Size2D};

/// Return the tiles on a tile sheet image.
///
/// Tiles are bounding boxes of non-background pixel groups surrounded by only background pixels or
/// image edges. Background color is the color of the bottom right corner pixel of the image. The
/// bounding boxes are returned lexically sorted by the coordinates of their bottom right corners,
/// first along the y-axis then along the x-axis. This produces a natural left-to-right,
/// bottom-to-top ordering for a cleanly laid out tile sheet.
///
/// Note that the background color is a solid color, not transparent pixels. The inner tiles may
/// have transparent parts, so a solid color is needed to separate them.
pub fn tilesheet_bounds<I>(image: &I) -> Vec<Rect<i32>>
    where I: GenericImage,
          I::Pixel: PartialEq
{
    let mut ret: Vec<Rect<i32>> = Vec::new();
    let image_rect = Rect::new(Point2D::new(0, 0),
                               Size2D::new(image.width() as i32, image.height() as i32));

    let background = image.get_pixel(image.width() - 1, image.height() - 1);
    for y in image_rect.min_y()..image_rect.max_y() {
        for x in image_rect.min_x()..image_rect.max_x() {
            let pt = Point2D::new(x, y);
            // Skip areas that are already contained in known tiles.
            if ret.iter().any(|&r| r.contains(&pt)) {
                continue;
            }

            if image.get_pixel(pt.x as u32, pt.y as u32) != background {
                ret.push(tile_bounds(image, pt, background));
            }
        }
    }

    ret.sort_by(|a, b| rect_key(a).cmp(&rect_key(b)));
    return ret;

    fn rect_key(x: &Rect<i32>) -> (i32, i32) {
        (x.bottom_right().y, x.bottom_right().x)
    }
}

/// Find the smallest bounding box around seed pixel whose sides are either all background color or
/// image edge.
fn tile_bounds<I>(image: &I, seed_pos: Point2D<i32>, background: I::Pixel) -> Rect<i32>
    where I: GenericImage,
          I::Pixel: PartialEq
{
    let image_rect = Rect::new(Point2D::new(0, 0),
                               Size2D::new(image.width() as i32, image.height() as i32));
    let mut ret = Rect::new(seed_pos, Size2D::new(1, 1));

    // How many consecutive edges couldn't be expanded. Once this hits 4, the tile is complete.
    let mut unchanged_count = 0;

    while unchanged_count < 4 {
        for dir in 0..4 {

            // Try adding a 1-pixel wide strip to the tile rectangle.
            let new_area = match dir {
                0 => {
                    Rect::new(ret.origin + Point2D::new(0, -1),
                              Size2D::new(ret.size.width, 1))
                }
                1 => Rect::new(ret.top_right(), Size2D::new(1, ret.size.height)),
                2 => Rect::new(ret.bottom_left(), Size2D::new(ret.size.width, 1)),
                3 => {
                    Rect::new(ret.origin + Point2D::new(-1, 0),
                              Size2D::new(1, ret.size.height))
                }
                _ => panic!("Bad dir {}", dir),
            };

            // The new area is outside image bounds, cannot add.
            if !image_rect.contains_rect(&new_area) {
                unchanged_count += 1;
                continue;
            }

            // The new area is all background pixels, the tile ends here.
            if (new_area.min_x()..new_area.max_x())
                   .zip(new_area.min_y()..new_area.max_y())
                   .all(|(x, y)| image.get_pixel(x as u32, y as u32) == background) {
                unchanged_count += 1;
                continue;
            }


            // Otherwise good to go, add the new bit.
            ret = ret.union(&new_area);
            unchanged_count = 0;
        }
    }

    ret
}
