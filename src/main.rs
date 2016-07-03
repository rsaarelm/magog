extern crate euclid;
#[macro_use]
extern crate glium;
extern crate image;
extern crate vitral;

mod backend;

pub use euclid::Point2D;
pub use glium::{glutin, DisplayBuild};
pub use backend::Backend;

type Color = [f32; 4];

type ImageRef = usize;

type Splat = Vec<(ImageRef, Point2D<f32>, Color)>;

pub fn main() {
    let display = glutin::WindowBuilder::new()
                      .build_glium()
                      .unwrap();

    let mut backend = Backend::new(&display);

    // Construct Vitral context.
    let mut context: backend::Context;
    let mut builder = vitral::Builder::new();
    context = builder.build(|img| backend.make_texture(&display, img));

    let font = context.default_font();

    loop {
        context.begin_frame();

        context.draw_text(font, Point2D::new(4.0, 20.0), [1.0, 1.0, 1.0, 1.0], "Hello, world!");

        if !backend.update(&display, &mut context) {
            return;
        }
    }
}
