#[feature(globs)];
extern crate cgmath;
extern crate glutil;
extern crate color;
extern crate calx;
extern crate stb;
extern crate collections;

use glutil::app::App;
use glutil::key;
use glutil::atlas::Sprite;
use cgmath::vector::{Vec2};
use stb::image::Image;
use roamy::Roamy;

pub mod fov;
pub mod area;
pub mod areaview;
pub mod dijkstra;
pub mod roamy;
pub mod mapgen;

pub fn main() {
    let mut app = App::new(640, 360, "Mapgen demo");
    let tiles = Image::load("assets/tile.png", 1).unwrap();
    let sprites = Sprite::new_alpha_set(
        &Vec2::new(32, 32),
        &Vec2::new(tiles.width as int, tiles.height as int),
        tiles.pixels,
        &Vec2::new(-16, -16));
    for i in range(0,16) {
        app.add_sprite(~sprites[i].clone());
    }

    let mut state = Roamy::new();

    while app.alive {
        state.draw(&mut app);

        app.flush();

        for key in app.key_buffer().iter() {
            if key.code == key::ESC {
                return;
            }
        }
    }
}

