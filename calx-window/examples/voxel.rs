extern crate glium;
extern crate calx_color;
extern crate calx_system;
extern crate calx_window;
extern crate cgmath;
extern crate collision;

use calx_window::{WindowBuilder, Event, Key, Window};
use calx_color::{SRgba, Rgba};
use collision::Intersect;
use cgmath::{EuclideanVector, Matrix, Point};

struct Frame {
    pub pixels: Vec<u32>,
}

impl Frame {
    pub fn new() -> Frame {
        let mut pixels = Vec::with_capacity(320 * 180);
        pixels.resize(320 * 180, 0x00000000);
        Frame {
            pixels: pixels
        }
    }



    pub fn blit(self, window: &mut Window) {
        window.set_frame(glium::texture::RawImage2d::from_raw_rgba(self.pixels, (320, 180)));
    }
}

fn main() {
    let mut window = WindowBuilder::new().set_size([320, 180]).set_title("Voxel demo").build();
    let mut tick = 0;

    loop {
        for e in window.events().into_iter() {
            match e {
                Event::Quit => return,
                Event::KeyPress(Key::Escape) => return,
                Event::KeyPress(Key::F12) => {
                    calx_system::save_screenshot("calx", window.screenshot())
                        .unwrap();
                }
                _ => (),
            }
        }

        let projection = cgmath::Matrix4::from(
            cgmath::PerspectiveFov {
                fovy: cgmath::deg(45.0),
                aspect: 16.0 / 9.0, // XXX: Hardcoding
                near: 0.1,
                far: 1024.0,
            });

        /*
        let a = (tick as f32) / 96.0;
        let eye =  cgmath::Point3::new(15.0 * a.sin(),
            -15.0 * a.cos(),
            0.0);
            */
        let eye = cgmath::Point3::new(0.0, -20.0, 0.0);
        let modelview: cgmath::Matrix4<f32> =
            cgmath::Matrix4::look_at(
                    eye,
                    cgmath::Point3::new(0.0, 0.0, 0.0),
                    cgmath::vec3(0.0, 0.0, 1.0));

        let mut frame = Frame::new();

        let orb = collision::Sphere {
            center: cgmath::Point3::new(0.0, 0.0, 0.0),
            radius: 1.0,
        };

        for y in 0..180 {
            for x in 0..320 {
                let u = (x as f32 - 160.0) / 160.0;
                let v = -(y as f32 - 90.0) / 90.0;

                let ray = collision::Ray::new(
                        eye,
                        projection.mul_m(&modelview).mul_v(cgmath::vec4(u, v, 1.0, 0.0)).truncate().normalize());

                if let Some(p) = (orb, ray).intersection() {
                    let normal = p.sub_p(orb.center).normalize();
                    let mut n = cgmath::dot(normal, cgmath::vec3(-1.0, -1.0, -1.0).normalize());
                    if n < 0.1 {
                        n = 0.1;
                    }
                    let c: SRgba = Rgba::new(n, n, 0.0, 0.0).into();
                    frame.pixels[x + 320 * y] = ((c.a as u32) << 24) + ((c.b as u32) << 16) + ((c.g as u32) << 8) + c.r as u32;
                }
            }
        }

        frame.blit(&mut window);

        window.end_frame();

        tick += 1;
    }
}
