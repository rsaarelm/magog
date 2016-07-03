extern crate euclid;
#[macro_use]
extern crate glium;
extern crate image;
extern crate vitral;

use euclid::Point2D;

mod backend;

type Color = [f32; 4];

type ImageRef = usize;

type Splat = Vec<(ImageRef, Point2D<f32>, Color)>;

pub fn main() {
    println!("Hello, world!");
}
