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
        let mut pixels = Vec::with_capacity(640 * 360);
        pixels.resize(640 * 360, 0x00000000);
        Frame {
            pixels: pixels
        }
    }



    pub fn blit(self, window: &mut Window) {
        window.set_frame(glium::texture::RawImage2d::from_raw_rgba(self.pixels, (640, 360)));
    }
}

fn main() {
    let mut window = WindowBuilder::new().set_title("Voxel demo").build();
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
                aspect: 640.0 / 380.0, // XXX: Hardcoding
                near: 0.1,
                far: 1024.0,
            });

        let a = (tick as f32) / 96.0;
        let modelview: cgmath::Matrix4<f32> =
            cgmath::Matrix4::look_at(cgmath::Point3::new(15.0 * a.sin() +
                                                         16.0,
                                                         15.0 * a.cos() +
                                                         16.0,
                                                         16.0),
                                     cgmath::Point3::new(0.0, 0.0, 0.0),
                                     cgmath::vec3(0.0, 0.0, 1.0));

        let mut frame = Frame::new();

        let orb = collision::Sphere {
            center: cgmath::Point3::new(0.0, 20.0, 0.0),
            radius: 1.0,
        };

        for y in 0..360 {
            for x in 0..640 {
                let u = (x as f32 - 320.0) / 320.0;
                let v = (y as f32 - 180.0) / 180.0;

                let ray = collision::Ray::new(
                        cgmath::Point3::new(0.0, 0.0, 0.0),
                        projection./*mul_m(&modelview).*/mul_v(cgmath::vec4(u, 1.0, v, 1.0)).truncate().normalize());

                if let Some(p) = (orb, ray).intersection() {
                    let normal = p.sub_p(orb.center).normalize();
                    let mut n = cgmath::dot(normal, cgmath::vec3(-1.0, -1.0, -1.0).normalize());
                    if n < 0.1 {
                        n = 0.1;
                    }
                    let c: SRgba = Rgba::new(n, n, 0.0, 0.0).into();
                    frame.pixels[x + 640 * y] = ((c.a as u32) << 24) + ((c.b as u32) << 16) + ((c.g as u32) << 8) + c.r as u32;
                }

                //println!("{:?}", projection * cgmath::vec4(u, 1.0, v, 1.0));
            }
        }

        frame.blit(&mut window);

        window.end_frame();

        tick += 1;
    }
}
