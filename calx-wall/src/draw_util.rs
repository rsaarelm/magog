use cgmath::{Vector2, vec2, dot};
use calx_color::{color, Rgba};
use calx_layout::Rect;
use calx_layout::Anchor::*;
use wall::{Wall, Vertex};

/// Helper methods for render context that do not depend on the underlying
/// implementation details.
pub trait DrawUtil {
    /// Draw a thick solid line on the canvas.
    fn draw_line<C, V>(&mut self, width: f32, p1: V, p2: V, layer: f32, color: C)
        where C: Into<Rgba>+Copy,
              V: Into<[f32; 2]>;

    /// Get the size of an atlas image.
    fn image_dim(&self, img: usize) -> [u32; 2];

    /// Draw a stored image on the canvas.
    fn draw_image<C, D, V>(&mut self, img: usize, offset: V, z: f32, color: C, back_color: D)
        where C: Into<Rgba>+Copy,
              D: Into<Rgba>+Copy,
              V: Into<[f32; 2]>;

    /// Draw a filled rectangle.
    fn fill_rect<C: Into<Rgba>+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: C);

    /// Draw a wireframe rectangle.
    fn draw_rect<C: Into<Rgba>+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: C);
}

impl DrawUtil for Wall {
    fn draw_line<C, V>(&mut self, width: f32, p1: V, p2: V, layer: f32, color: C)
        where C: Into<Rgba>+Copy,
              V: Into<[f32; 2]>
    {
        let p1: Vector2<f32> = Vector2::from(p1.into());
        let p2: Vector2<f32> = Vector2::from(p2.into());

        if p1 == p2 { return; }

        let tex = self.tiles[0].tex.top;

        // The front vector. Extend by width.
        let v1 = p2 - p1;
        let scalar = dot(v1, v1);
        let scalar = (scalar + width * width) / scalar;
        let v1 = v1 * scalar;

        // The sideways vector, turn into unit vector, then multiply by half the width.
        let v2 = vec2(-v1[1], v1[0]);
        let scalar = width / 2.0 * 1.0 / dot(v2, v2).sqrt();
        let v2 = v2 * scalar;

        let color: Rgba = color.into();
        self.add_mesh(
            vec![
            Vertex::new(p1 + v2, layer, tex, color, color::BLACK),
            Vertex::new(p1 - v2, layer, tex, color, color::BLACK),
            Vertex::new(p1 - v2 + v1, layer, tex, color, color::BLACK),
            Vertex::new(p1 + v2 + v1, layer, tex, color, color::BLACK),
            ],
            vec![[0, 1, 2], [0, 2, 3]]);
    }

    fn image_dim(&self, img: usize) -> [u32; 2] {
        let size = self.tiles[img].pos.size;
        [size[0] as u32, size[1] as u32]
    }

    fn draw_image<C, D, V>(&mut self, img: usize, offset: V, z: f32, color: C, back_color: D)
        where C: Into<Rgba>+Copy,
              D: Into<Rgba>+Copy,
              V: Into<[f32; 2]> {
        // Use round numbers, fractions seem to cause artifacts to pixels.
        let mut offset = offset.into();
        offset[0] = offset[0].floor();
        offset[1] = offset[1].floor();

        let mut pos;
        let tex;
        {
            let data = self.tiles[img];
            pos = data.pos;
            pos.top[0] += offset[0];
            pos.top[1] += offset[1];
            tex = data.tex;
        }

        let color: Rgba = color.into();
        let back_color: Rgba = back_color.into();
        self.add_mesh(
            vec![
            Vertex::new(pos.point(TopLeft), z, tex.point(TopLeft), color, back_color),
            Vertex::new(pos.point(TopRight), z, tex.point(TopRight), color, back_color),
            Vertex::new(pos.point(BottomRight), z, tex.point(BottomRight), color, back_color),
            Vertex::new(pos.point(BottomLeft), z, tex.point(BottomLeft), color, back_color),
            ],
            vec![[0, 1, 2], [0, 2, 3]]);
    }

    fn fill_rect<C: Into<Rgba>+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: C) {
        let tex = self.tiles[0].tex.top;

        let color: Rgba = color.into();
        self.add_mesh(
            vec![
            Vertex::new(rect.point(TopLeft), z, tex, color, color::BLACK),
            Vertex::new(rect.point(TopRight), z, tex, color, color::BLACK),
            Vertex::new(rect.point(BottomRight), z, tex, color, color::BLACK),
            Vertex::new(rect.point(BottomLeft), z, tex, color, color::BLACK),
            ],
            vec![[0, 1, 2], [0, 2, 3]]);
    }

    fn draw_rect<C: Into<Rgba>+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: C) {
        self.draw_line(1.0, Vector2::from(rect.point(TopLeft)), Vector2::from(rect.point(TopRight)) - vec2(1.0, 0.0), z, color);
        self.draw_line(1.0, Vector2::from(rect.point(TopRight)) - vec2(1.0, 0.0), Vector2::from(rect.point(BottomRight)) - vec2(1.0, 0.0), z, color);
        self.draw_line(1.0, Vector2::from(rect.point(BottomLeft)) - vec2(0.0, 1.0), Vector2::from(rect.point(BottomRight)) - vec2(1.0, 1.0), z, color);
        self.draw_line(1.0, rect.point(TopLeft), rect.point(BottomLeft), z, color);
    }

    /*
    fn draw_char<C: Into<Rgba>+Copy, D: Into<Rgba>+Copy>(&mut self, c: char, offset: V2<f32>, z: f32, color: C, border: Option<D>) {
        static BORDER: [V2<f32>; 8] =
            [V2(-1.0, -1.0), V2( 0.0, -1.0), V2( 1.0, -1.0),
             V2(-1.0,  0.0),                 V2( 1.0,  0.0),
             V2(-1.0,  1.0), V2( 0.0,  1.0), V2( 1.0,  1.0)];
        if let Some(img) = self.font_image(c) {
            if let Some(b) = border {
                // Put the border a tiny bit further in the z-buffer so it
                // won't clobber the text on the same layer.
                let border_z = z + 0.00001;
                for &d in BORDER.iter() {
                    self.draw_image(img, offset + d, border_z, b, color::BLACK);
                }
            }
            self.draw_image(img, offset, z, color, color::BLACK);
        }
    }

    fn char_width(&self, c: char) -> f32 {
        // Infer letter width from the cropped atlas image. (Use mx instead of
        // dim on the pos rectangle so that the left-side space will be
        // preserved and the letters are kept slightly apart.)
        if let Some(img) = self.font_image(c) {
            let width = self.tiles[img].pos.mx().0;
            return width;
        }

        // Not a valid letter.
        (FONT_W / 2) as f32
    }
    */

    /*
    fn save_screenshot(&mut self, basename: &str) {
        use time;
        use std::path::{Path};
        use std::fs::{self, File};
        use image;

        let shot = self.screenshot();

        let timestamp = time::precise_time_s() as u64;
        // Create screenshot filenames by concatenating the current timestamp in
        // seconds with a running number from 00 to 99. 100 shots per second
        // should be good enough.

        // Default if we fail to generate any of the 100 candidates for this
        // second, just overwrite with the "xx" prefix then.
        let mut filename = format!("{}-{}{}.png", basename, timestamp, "xx");

        // Run through candidates for this second.
        for i in 0..100 {
            let test_filename = format!("{}-{}{:02}.png", basename, timestamp, i);
            // If file does not exist.
            if fs::metadata(&test_filename).is_err() {
                // Thread-safe claiming: create_dir will fail if the dir
                // already exists (it'll exist if another thread is gunning
                // for the same filename and managed to get past us here).
                // At least assuming that create_dir is atomic...
                let squat_dir = format!(".tmp-{}{:02}", timestamp, i);
                if fs::create_dir(&squat_dir).is_ok() {
                    File::create(&test_filename).unwrap();
                    filename = test_filename;
                    fs::remove_dir(&squat_dir).unwrap();
                    break;
                } else {
                    continue;
                }
            }
        }

        let _ = image::save_buffer(&Path::new(&filename), &shot, shot.width(), shot.height(), image::ColorType::RGB(8));
    }
    */
}
