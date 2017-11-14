extern crate euclid;
extern crate image;

#[cfg(feature = "glium_backend")]
extern crate glium;
extern crate vitral;

fn main() { feature::main(); }

#[cfg(feature = "glium_backend")]
mod feature {
    use euclid::{Rect, Point2D, point2, Size2D, vec2};
    use glium::glutin;
    use image;
    use std::path::Path;
    use vitral::{self, Align, ButtonAction, RectUtil};
    use vitral::glium_backend as backend;
    use vitral::glium_backend::{DefaultVertex, TextureHandle};

    type Core = vitral::Core<TextureHandle, DefaultVertex>;
    type Backend = backend::Backend<DefaultVertex>;
    type ImageData = vitral::ImageData<TextureHandle>;

    pub fn clamp<C: PartialOrd + Copy>(mn: C, mx: C, x: C) -> C {
        if x < mn {
            mn
        } else if x > mx {
            mx
        } else {
            x
        }
    }

    fn load_image(backend: &mut Backend, path: &str) -> ImageData {
        let image: vitral::ImageBuffer = image::open(&Path::new(path)).unwrap().into();
        let size = image.size;
        let texture = backend.make_texture(image);

        vitral::ImageData {
            texture,
            size,
            tex_coords: Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1.0, 1.0)),
        }
    }

    struct App {
        core: Core,
        font: vitral::FontData<TextureHandle>,
        image: vitral::ImageData<TextureHandle>,
        fore_color: [f32; 4],
        back_color: [f32; 4],
        bounds: Rect<f32>,
    }

    impl App {
        pub fn new(core: Core, backend: &mut Backend) -> App {
            let bounds = core.bounds();
            let font = vitral::build_default_font(|img| backend.make_texture(img));
            let image = load_image(backend, "julia.png");
            App {
                core,
                font,
                image,
                fore_color: [1.0, 0.5, 0.1, 1.0],
                back_color: [0.0, 0.0, 0.0, 1.0],
                bounds,
            }
        }

        fn bright_color(&self) -> [f32; 4] {
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
            self.core.fill_rect(
                &bounds.inflate(-1.0, -1.0),
                self.back_color,
            );

            let inner = bounds.inflate(-3.0, -3.0).inclusivize();

            self.core.draw_line(
                1.0,
                color,
                inner.bottom_right(),
                inner.origin,
            );

            self.core.draw_line(
                1.0,
                color,
                inner.top_right(),
                inner.bottom_left(),
            );

            self.core.click_state(bounds) == ButtonAction::LeftClicked
        }

        fn render(&mut self) {
            self.core.draw_image(
                &self.image,
                point2(20.0, 20.0),
                [1.0, 1.0, 1.0, 1.0],
            );
            self.outline_text(point2(22.0, 22.0), "Hello, world!");
        }
    }

    pub fn main() {
        let size = Size2D::new(640.0, 360.0);
        let mut backend: Backend = backend::start(
            size.width as u32,
            size.height as u32,
            "Vitral Demo",
            backend::DEFAULT_SHADER,
        ).expect("Failed to start Glium backend!");

        let core = vitral::Builder::new().build(size, |img| backend.make_texture(img));
        let mut app = App::new(core, &mut backend);

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
                if k.state == glutin::ElementState::Released {
                    continue;
                }

                if k.key_code == glutin::VirtualKeyCode::Q {
                    return;
                }

                if k.key_code == glutin::VirtualKeyCode::F12 {
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
