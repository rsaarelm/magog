use std::convert::{Into};
use super::canvas::{Canvas, Image, FONT_W};
use super::{WidgetId};
use ::{V2, Rect, color, Rgba};
use ::Anchor::*;

/// Helper methods for canvas context that do not depend on the underlying
/// implementation details.
pub trait CanvasUtil {
    /// Draw a thick solid line on the canvas.
    fn draw_line<C: Into<Rgba>+Copy>(&mut self, width: f32, p1: V2<f32>, p2: V2<f32>, layer: f32, color: C);
    /// Get the size of an atlas image.
    fn image_dim(&self, img: Image) -> V2<u32>;

    /// Draw a stored image on the canvas.
    fn draw_image<C: Into<Rgba>+Copy, D: Into<Rgba>+Copy>(&mut self, img: Image,
        offset: V2<f32>, z: f32, color: C, back_color: D);

    /// Draw a filled rectangle.
    fn fill_rect<C: Into<Rgba>+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: C);

    /// Draw a wireframe rectangle.
    fn draw_rect<C: Into<Rgba>+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: C);

    /// Draw an immediate GUI button and return whether it was pressed.
    fn button(&mut self, id: WidgetId, pos: V2<f32>, z: f32) -> bool;

    fn draw_char<C: Into<Rgba>+Copy, D: Into<Rgba>+Copy>(&mut self, c: char, offset: V2<f32>, z: f32, color: C, border: Option<D>);

    fn char_width(&self, c: char) -> f32;

    /// Write a timestamped screenshot PNG to disk.
    fn save_screenshot(&mut self, basename: &str);
}

impl<'a> CanvasUtil for Canvas<'a> {
    fn draw_line<C: Into<Rgba>+Copy>(&mut self, width: f32, p1: V2<f32>, p2: V2<f32>, layer: f32, color: C) {
        if p1 == p2 { return; }

        let tex = self.solid_tex_coord();

        // The front vector. Extend by width.
        let v1 = p2 - p1;
        let scalar = v1.dot(v1);
        let scalar = (scalar + width * width) / scalar;
        let v1 = v1 * scalar;

        // The sideways vector, turn into unit vector, then multiply by half the width.
        let v2 = V2(-v1.1, v1.0);
        let scalar = width / 2.0 * 1.0 / v2.dot(v2).sqrt();
        let v2 = v2 * scalar;

        let ind0 = self.num_vertices();
        self.push_vertex(p1 + v2, layer, tex, color, color::BLACK);
        self.push_vertex(p1 - v2, layer, tex, color, color::BLACK);
        self.push_vertex(p1 - v2 + v1, layer, tex, color, color::BLACK);
        self.push_vertex(p1 + v2 + v1, layer, tex, color, color::BLACK);
        self.push_triangle(ind0, ind0 + 1, ind0 + 2);
        self.push_triangle(ind0, ind0 + 2, ind0 + 3);

        self.flush();
    }

    fn image_dim(&self, img: Image) -> V2<u32> {
        self.image_data(img).pos.1.map(|x| x as u32)
    }

    fn draw_image<C: Into<Rgba>+Copy, D: Into<Rgba>+Copy>(&mut self, img: Image,
        offset: V2<f32>, z: f32, color: C, back_color: D) {
        // Use round numbers, fractions seem to cause artifacts to pixels.
        let offset = offset.map(|x| x.floor());
        let mut pos;
        let mut tex;
        {
            let data = self.image_data(img);
            pos = data.pos + offset;
            tex = data.tex;
        }

        let ind0 = self.num_vertices();

        self.push_vertex(pos.point(TopLeft), z, tex.point(TopLeft), color, back_color);
        self.push_vertex(pos.point(TopRight), z, tex.point(TopRight), color, back_color);
        self.push_vertex(pos.point(BottomRight), z, tex.point(BottomRight), color, back_color);
        self.push_vertex(pos.point(BottomLeft), z, tex.point(BottomLeft), color, back_color);

        self.push_triangle(ind0, ind0 + 1, ind0 + 2);
        self.push_triangle(ind0, ind0 + 2, ind0 + 3);

        self.flush();
    }

    fn fill_rect<C: Into<Rgba>+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: C) {
        let tex = self.solid_tex_coord();
        let ind0 = self.num_vertices();

        self.push_vertex(rect.point(TopLeft), z, tex, color, color::BLACK);
        self.push_vertex(rect.point(TopRight), z, tex, color, color::BLACK);
        self.push_vertex(rect.point(BottomRight), z, tex, color, color::BLACK);
        self.push_vertex(rect.point(BottomLeft), z, tex, color, color::BLACK);

        self.push_triangle(ind0, ind0 + 1, ind0 + 2);
        self.push_triangle(ind0, ind0 + 2, ind0 + 3);

        self.flush();
    }

    fn draw_rect<C: Into<Rgba>+Copy>(&mut self, rect: &Rect<f32>, z: f32, color: C) {
        self.draw_line(1.0, rect.point(TopLeft), rect.point(TopRight) - V2(1.0, 0.0), z, color);
        self.draw_line(1.0, rect.point(TopRight) - V2(1.0, 0.0), rect.point(BottomRight) - V2(1.0, 0.0), z, color);
        self.draw_line(1.0, rect.point(BottomLeft) - V2(0.0, 1.0), rect.point(BottomRight) - V2(1.0, 1.0), z, color);
        self.draw_line(1.0, rect.point(TopLeft), rect.point(BottomLeft), z, color);
    }

    fn button(&mut self, id: WidgetId, pos: V2<f32>, z: f32) -> bool {
        // TODO: Button visual style! Closures?
        let area = Rect(pos, V2(64.0, 16.0));
        let mut color = color::GREEN;
        if area.contains(&self.mouse_pos) {
            self.hot_widget = Some(id);
            if self.active_widget.is_none() && self.mouse_pressed {
                self.active_widget = Some(id);
            }
            color = color::RED;
        }

        self.fill_rect(&area, z, color);

        return !self.mouse_pressed // Mouse is released
            && self.active_widget == Some(id) // But this button is hot and active
            && self.hot_widget == Some(id);
    }

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
        // Special case for space, the atlas image won't have a width.
        if c == ' ' { return (FONT_W / 2) as f32; }

        // Infer letter width from the cropped atlas image. (Use mx instead of
        // dim on the pos rectangle so that the left-side space will be
        // preserved and the letters are kept slightly apart.)
        if let Some(img) = self.font_image(c) {
            let width = self.image_data(img).pos.mx().0;
            return width;
        }

        // Not a valid letter.
        (FONT_W / 2) as f32
    }

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
}
