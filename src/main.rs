// Don't show a console window when running on Windows.
#![windows_subsystem = "windows"]

extern crate rand;
extern crate euclid;
extern crate glium;
extern crate scancode;
extern crate vitral;
extern crate calx_grid;
#[macro_use]
extern crate calx_alg;
extern crate world;
extern crate display;

pub mod game_view;

use euclid::{Point2D, Rect, Size2D};
use game_view::View;
use glium::{DisplayBuild, glutin};
use std::fs::File;
use vitral::Context;
use world::{Location, Mutate, World};

pub fn main() {
    // Construct display and Vitral context.
    let glium = glutin::WindowBuilder::new().build_glium().unwrap();

    let screen_area = Rect::new(Point2D::new(0.0, 0.0), Size2D::new(640.0f32, 360.0f32));
    let mut backend = display::Backend::new(&glium,
                                            screen_area.size.width as u32,
                                            screen_area.size.height as u32);

    let seed = 1;

    let mut world = World::new(seed);

    /// TODO error handling.
    let prefab = world::load_prefab(&mut File::open("sprint.toml").unwrap()).unwrap();
    world.deploy_prefab(Location::new(-21, -22, 0), &prefab);

    let mut view = View::new(world);

    loop {
        backend.begin_frame();
        view.draw(&mut backend, &screen_area);
        if !backend.update(&glium) {
            return;
        }
    }
}
