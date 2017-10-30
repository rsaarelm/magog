extern crate euclid;
extern crate image;

extern crate glium;
extern crate vitral;
extern crate vitral_glium;

use euclid::{Rect, Point2D, point2, Size2D, rect, vec2};
use glium::glutin;
use image::{GenericImage, Pixel};
use std::path::Path;
use vitral::{Align, ButtonAction, RectUtil};
use vitral_glium::{Backend, DefaultVertex, TextureHandle};

type Core = vitral::Core<TextureHandle, DefaultVertex>;

pub fn clamp<C: PartialOrd + Copy>(mn: C, mx: C, x: C) -> C {
    if x < mn {
        mn
    } else if x > mx {
        mx
    } else {
        x
    }
}

fn _load_image<V>(backend: &mut vitral_glium::Backend<V>, path: &str) -> vitral::ImageData<usize>
where
    V: vitral::Vertex + glium::Vertex,
{
    let image = image::open(&Path::new(path)).unwrap();
    let (w, h) = image.dimensions();
    let pixels = image
        .pixels()
        .map(|(_, _, p)| {
            let (r, g, b, a) = p.channels4();
            r as u32 + ((g as u32) << 8) + ((b as u32) << 16) + ((a as u32) << 24)
        })
        .collect();
    let image = vitral::ImageBuffer {
        size: Size2D::new(w, h),
        pixels,
    };

    let id = backend.make_texture(image);

    vitral::ImageData {
        texture: id,
        size: Size2D::new(w, h),
        tex_coords: Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1.0, 1.0)),
    }
}

struct App {
    core: Core,
    font: vitral::FontData<TextureHandle>,
    fore_color: [f32; 4],
    back_color: [f32; 4],
    bounds: Rect<f32>,
}

impl App {
    pub fn new(core: Core, font: vitral::FontData<TextureHandle>) -> App {
        let bounds = core.bounds();
        App {
            core,
            font,
            fore_color: [1.0, 0.5, 0.1, 1.0],
            back_color: [0.0, 0.0, 0.0, 1.0],
            bounds
        }
    }

    fn bright_color(&self) -> [f32; 4] {
        [clamp(0.0, 1.0, self.fore_color[0] * 1.5),
         clamp(0.0, 1.0, self.fore_color[1] * 1.5),
         clamp(0.0, 1.0, self.fore_color[2] * 1.5),
         1.0]
    }

    fn begin(&mut self) { self.core.begin_frame(); }

    fn outline_text(&mut self, pos: Point2D<f32>, text: &str) -> Point2D<f32> {
        for offset in &[
            vec2(-1.0, 0.0),
            vec2(1.0, 0.0),
            vec2(0.0, -1.0),
            vec2(0.0, 1.0),
        ]
        {
            self.core.draw_text(
                &self.font,
                pos + *offset,
                Align::Left,
                self.back_color,
                text,
            );
        }

        self.core.draw_text(
            &self.font,
            pos,
            Align::Left,
            self.fore_color,
            text,
        )
    }

    fn title_bar(&mut self, bounds: &Rect<f32>, text: &str) {
        self.core.fill_rect(bounds, self.back_color);
        {
            let bounds = bounds.inclusivize();
            self.core.draw_line(
                1.0,
                self.fore_color,
                bounds.bottom_left(),
                bounds.bottom_right());
        }

        // Margin
        let bounds = bounds.inflate(-2.0, -2.0);

        self.core.draw_text(
            &self.font,
            bounds.anchor(&point2(0.0, -1.0)),
            Align::Center,
            self.fore_color,
            text);
    }

    fn quit_button(&mut self, bounds: &Rect<f32>) -> bool {
        let click_state = self.core.click_state(bounds);

        let color = if click_state != ButtonAction::Inert {
            self.bright_color()
        } else {
            self.fore_color
        };

        self.core.fill_rect(bounds, color);
        self.core.fill_rect(&bounds.inflate(-1.0, -1.0), self.back_color);

        let inner = bounds.inflate(-3.0, -3.0).inclusivize();

        self.core.draw_line(
            1.0,
            color,
            inner.bottom_right(),
            inner.origin);

        self.core.draw_line(
            1.0,
            color,
            inner.top_right(),
            inner.bottom_left());

        self.core.click_state(bounds) == ButtonAction::LeftClicked
    }

    fn render(&mut self) {
        self.core.fill_rect(
            &rect(0.0, 20.0, 128.0, 128.0),
            [0.4, 0.5, 0.5, 1.0],
        );
        self.outline_text(point2(10.0, 30.0), "Hello, world!");
    }
}

fn main() {
    let size = Size2D::new(640.0, 360.0);
    let mut backend: Backend<DefaultVertex> = vitral_glium::start(
        size.width as u32,
        size.height as u32,
        "Vitral Demo",
        vitral_glium::DEFAULT_SHADER,
    ).expect("Failed to start Glium backend!");

    let core = vitral::Builder::new().build(size, |img| backend.make_texture(img));
    let font = vitral::build_default_font(|img| backend.make_texture(img));
    let mut app = App::new(core, font);

    loop {
        app.begin();
        app.render();
        let (_, title_area) = app.bounds.horizontal_split(12.0);
        app.title_bar(&title_area, "Vitral Demo");

        let (_, widget_area) = title_area.vertical_split(-12.0);
        if app.quit_button(&widget_area) {
            return;
        }

        while let Some(k) = backend.poll_key() {
            if k.key_code == glutin::VirtualKeyCode::Q {
                return;
            }
        }

        if !backend.update(&mut app.core) {
            break;
        }
    }
}
