extern crate euclid;
extern crate image;
extern crate glium;

extern crate vitral;
extern crate vitral_glium;

use std::path::Path;
use glium::{DisplayBuild, glutin};
use euclid::Point2D;
use vitral_glium::{Backend, DefaultVertex};

fn main() {
    // Construct Glium backend.
    let display = glutin::WindowBuilder::new()
                      .build_glium()
                      .unwrap();

    let mut backend = Backend::new(vitral_glium::default_program(&display).unwrap());

    // Construct Vitral context.
    let mut context: vitral::Context<usize, DefaultVertex>;
    let mut builder = vitral::Builder::new();
    let image = builder.add_image(&image::open(&Path::new("julia.png")).unwrap());
    context = builder.build(|img| backend.make_texture(&display, img));

    let font = 0;

    let mut test_input = String::new();

    // Run the program.
    loop {
        context.begin_frame();

        context.draw_image(image, Point2D::new(100.0, 100.0), [1.0, 1.0, 1.0, 1.0]);

        if context.button("Hello, world") {
            println!("Click");
        }

        if context.button("Another button") {
            println!("Clack {}", test_input);
        }

        context.text_input(font,
                           Point2D::new(10.0, 120.0),
                           [0.8, 0.8, 0.8, 1.0],
                           &mut test_input);

        if !backend.update(&display, &mut context) {
            return;
        }
    }
}
