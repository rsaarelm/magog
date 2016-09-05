extern crate euclid;
extern crate image;
extern crate glium;

extern crate vitral;
extern crate vitral_glium;

use std::path::Path;
use image::GenericImage;
use glium::{DisplayBuild, glutin};
use euclid::{Rect, Point2D, Size2D};
use vitral_glium::{Backend, DefaultVertex};

fn load_image<V>(display: &glium::Display,
                 backend: &mut Backend<V>,
                 path: &str)
                 -> vitral::ImageData<usize>
    where V: vitral::Vertex + glium::Vertex
{
    let image = image::open(&Path::new(path)).unwrap();
    let (w, h) = image.dimensions();
    let pixels = image.pixels()
                      .map(|(_, _, p)| unsafe { ::std::mem::transmute::<image::Rgba<u8>, u32>(p) })
                      .collect();
    let image = vitral::ImageBuffer {
        width: w,
        height: h,
        pixels: pixels,
    };

    let id = backend.make_texture(display, image);

    vitral::ImageData {
        texture: id,
        size: Size2D::new(w, h),
        tex_coords: Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1.0, 1.0)),
    }
}

fn main() {
    // Construct Glium backend.
    let display = glutin::WindowBuilder::new()
                      .build_glium()
                      .unwrap();

    let mut backend = Backend::new(&display,
                                   vitral_glium::default_program(&display).unwrap(),
                                   640,
                                   360);

    // Construct Vitral context.
    let mut context: vitral::Context<usize, DefaultVertex>;
    let builder = vitral::Builder::new();
    let image = load_image(&display, &mut backend, "julia.png");
    context = builder.build(|img| backend.make_texture(&display, img));

    let mut test_input = String::new();

    // Run the program.
    loop {
        context.begin_frame();

        context.draw_image(&image, Point2D::new(100.0, 100.0), [1.0, 1.0, 1.0, 1.0]);

        if context.button("Hello, world") {
            println!("Click");
        }

        if context.button("Another button") {
            println!("Clack {}", test_input);
        }

        let font = context.default_font();
        context.text_input(&*font,
                           Point2D::new(10.0, 120.0),
                           [0.8, 0.8, 0.8, 1.0],
                           &mut test_input);

        if !backend.update(&display, &mut context) {
            return;
        }
    }
}
