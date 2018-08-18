extern crate euclid;
extern crate image;

#[cfg(feature = "glium_backend")]
extern crate glium;
extern crate vitral;

fn main() { feature::main(); }

#[cfg(feature = "glium_backend")]
mod feature {
    use euclid::{point2, vec2, Point2D, Rect, Size2D};
    use glium::glutin;
    use image;
    use vitral::glium_backend::DefaultVertex;
    use vitral::{self, Align, ButtonAction, Color, PngBytes, RectUtil};

    type Core = vitral::Core<DefaultVertex>;
    type Backend = vitral::glium_backend::Backend<DefaultVertex>;

    pub fn clamp<C: PartialOrd + Copy>(mn: C, mx: C, x: C) -> C {
        if x < mn {
            mn
        } else if x > mx {
            mx
        } else {
            x
        }
    }

    struct App {
        core: Core,
        atlas_cache: vitral::AtlasCache<String>,
        font: vitral::FontData,
        image: vitral::ImageData,
        fore_color: Color,
        back_color: Color,
        bounds: Rect<f32>,
    }

    impl App {
        pub fn new(core: Core, backend: &Backend) -> App {
            let bounds = core.bounds();
            let mut atlas_cache = vitral::AtlasCache::new(512, backend.texture_count());

            let font = atlas_cache.add_tilesheet_font(
                "font",
                PngBytes(include_bytes!("../tilesheet-font.png")),
                (32u8..128).map(|c| c as char),
            );

            let image = atlas_cache.add_sheet("julia", PngBytes(include_bytes!("../julia.png")));
            let image = atlas_cache.get(&image).clone();

            App {
                core,
                atlas_cache,
                font,
                image,
                fore_color: [1.0, 0.5, 0.1, 1.0],
                back_color: [0.0, 0.0, 0.0, 1.0],
                bounds,
            }
        }

        fn bright_color(&self) -> Color {
            [
                clamp(0.0, 1.0, self.fore_color[0] * 1.5),
                clamp(0.0, 1.0, self.fore_color[1] * 1.5),
                clamp(0.0, 1.0, self.fore_color[2] * 1.5),
                1.0,
            ]
        }

        fn begin(&mut self) { self.core.begin_frame(); }

        fn outline_text(&mut self, pos: Point2D<f32>, text: &str) -> Point2D<f32> {
            for offset in &[
                vec2(-1.0, 0.0),
                vec2(1.0, 0.0),
                vec2(0.0, -1.0),
                vec2(0.0, 1.0),
            ] {
                self.core.draw_text(
                    &self.font,
                    pos + *offset,
                    Align::Left,
                    self.back_color,
                    text,
                );
            }

            self.core
                .draw_text(&self.font, pos, Align::Left, self.fore_color, text)
        }

        fn title_bar(&mut self, bounds: &Rect<f32>, text: &str) {
            self.core.fill_rect(bounds, self.back_color);
            {
                let bounds = bounds.inclusivize();
                self.core.draw_line(
                    1.0,
                    self.fore_color,
                    bounds.bottom_left(),
                    bounds.bottom_right(),
                );
            }

            // Margin
            let bounds = bounds.inflate(-2.0, -2.0);

            self.core.draw_text(
                &self.font,
                bounds.anchor(&point2(0.0, -1.0)),
                Align::Center,
                self.fore_color,
                text,
            );
        }

        fn quit_button(&mut self, bounds: &Rect<f32>) -> bool {
            let click_state = self.core.click_state(bounds);

            let color = if click_state != ButtonAction::Inert {
                self.bright_color()
            } else {
                self.fore_color
            };

            self.core.fill_rect(bounds, color);
            self.core
                .fill_rect(&bounds.inflate(-1.0, -1.0), self.back_color);

            let inner = bounds.inflate(-3.0, -3.0).inclusivize();

            self.core
                .draw_line(1.0, color, inner.bottom_right(), inner.origin);

            self.core
                .draw_line(1.0, color, inner.top_right(), inner.bottom_left());

            self.core.click_state(bounds) == ButtonAction::LeftClicked
        }

        fn render(&mut self) {
            self.core
                .draw_image(&self.image, point2(20.0, 20.0), [1.0, 1.0, 1.0, 1.0]);
            self.outline_text(point2(22.0, 22.0), "Hello, world!");
        }
    }

    pub fn main() {
        let size = Size2D::new(640.0, 360.0);
        let mut backend: Backend = Backend::start(
            size.width as u32,
            size.height as u32,
            "Vitral Demo",
            vitral::glium_backend::DEFAULT_SHADER,
        ).expect("Failed to start Glium backend!");

        let core = vitral::Builder::new().build(size, |img| backend.make_texture(img));
        let mut app = App::new(core, &mut backend);

        loop {
            backend.sync_with_atlas_cache(&mut app.atlas_cache);
            app.begin();
            app.render();
            let (_, title_area) = app.bounds.horizontal_split(12.0);
            app.title_bar(&title_area, "Vitral Demo");

            let (_, widget_area) = title_area.vertical_split(-12.0);
            if app.quit_button(&widget_area) {
                return;
            }

            while let Some(k) = backend.poll_key() {
                if k.state == glutin::ElementState::Released {
                    continue;
                }

                if k.virtual_keycode == Some(glutin::VirtualKeyCode::Q) {
                    return;
                }

                if k.virtual_keycode == Some(glutin::VirtualKeyCode::F12) {
                    let screenshot: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
                        backend.screenshot().into();
                    image::save_buffer(
                        "screenshot.png",
                        &screenshot,
                        screenshot.width(),
                        screenshot.height(),
                        image::ColorType::RGB(8),
                    ).unwrap();
                    println!("Screenshot saved!");
                }
            }

            if !backend.update(&mut app.core) {
                break;
            }
        }
    }
}

#[cfg(not(feature = "glium_backend"))]
mod feature {
    pub fn main() {
        println!("Try running `cargo run --features \"glium_backend\" --example <example_name>");
    }
}
