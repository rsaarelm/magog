use euclid::{rect, vec2, Point2D};
use vitral::{Align, Color, Core, FontData, ImageData, Vertex};

/// Helpers for drawing into the local `Core` type.
pub trait DrawUtil {
    /// Draw an image with two-color vertices.
    fn draw_image_2color(
        &mut self,
        image: &ImageData,
        pos: Point2D<i32>,
        color: Color,
        back_color: Color,
    );

    /// Draw text with colored outline.
    fn draw_outline_text(
        &mut self,
        font: &FontData,
        pos: Point2D<i32>,
        align: Align,
        color: Color,
        back_color: Color,
        text: &str,
    ) -> Point2D<i32>;
}

impl DrawUtil for Core {
    fn draw_image_2color(
        &mut self,
        image: &ImageData,
        pos: Point2D<i32>,
        color: Color,
        back_color: Color,
    ) {
        self.start_texture(image.texture.clone());

        let area = rect(
            pos.x,
            pos.y,
            image.size.width as i32,
            image.size.height as i32,
        );

        let idx = self.push_raw_vertex(
            Vertex::new(area.origin.to_f32(), image.tex_coords.origin, color)
                .back_color(back_color),
        );
        self.push_raw_vertex(
            Vertex::new(
                area.top_right().to_f32(),
                image.tex_coords.top_right(),
                color,
            )
            .back_color(back_color),
        );
        self.push_raw_vertex(
            Vertex::new(
                area.bottom_right().to_f32(),
                image.tex_coords.bottom_right(),
                color,
            )
            .back_color(back_color),
        );
        self.push_raw_vertex(
            Vertex::new(
                area.bottom_left().to_f32(),
                image.tex_coords.bottom_left(),
                color,
            )
            .back_color(back_color),
        );

        self.push_triangle(idx, idx + 1, idx + 2);
        self.push_triangle(idx, idx + 2, idx + 3);
    }

    /// Draw text with colored outline.
    fn draw_outline_text(
        &mut self,
        font: &FontData,
        pos: Point2D<i32>,
        align: Align,
        color: Color,
        back_color: Color,
        text: &str,
    ) -> Point2D<i32> {
        for offset in &[vec2(-1, 0), vec2(1, 0), vec2(0, -1), vec2(0, 1)] {
            self.draw_text(font, pos + *offset, align, back_color, text);
        }

        self.draw_text(font, pos, align, color, text)
    }
}
