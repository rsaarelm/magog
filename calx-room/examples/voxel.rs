#[macro_use]
extern crate glium;
extern crate genmesh;
extern crate calx_system;
extern crate calx_window;

use calx_window::{WindowBuilder, Event, Key};

enum Face { East, North, Up, West, South, Down };

impl Face {
    pub fn normal(&self) -> [f32; 3] {
        match self {
            East =>  [1.0,  0.0,  0.0],
            North => [0.0,  1.0,  0.0],
            Up =>    [0.0,  0.0,  1.0],
            West =>  [-1.0, 0.0,  0.0],
            South => [0.0, -1.0,  0.0],
            Down =>  [0.0,  0.0, -1.0],
        }
    }
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3];
    pub color: [f32; 4],
}
implement_vertex!(Vertex, pos, normal, color);

fn main() {
    let mut window = WindowBuilder::new().set_title("Voxel demo").build();

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
        window.clear(0x7799DDFF);

        // window.display(&mut room);

        window.end_frame();
    }
}
