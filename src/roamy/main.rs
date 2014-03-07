#[feature(globs)];
extern crate cgmath;
extern crate glutil;
extern crate color;
extern crate calx;
extern crate stb;
extern crate collections;
extern crate num;

use glutil::app::App;
use glutil::key;
use glutil::atlas::Sprite;
use cgmath::vector::{Vec2};
use stb::image::Image;
use roamy::Roamy;
use area::DIRECTIONS6;

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
    for i in range(0, 64) {
        app.add_sprite(~sprites[i].clone());
    }

    let mut state = Roamy::new();

    while app.alive {
        state.draw(&mut app);

        for key in app.key_buffer().iter() {
            if key.code == key::ESC {
                return;
            }

            if key.code == key::SPACE {
                state.stop = !state.stop;
            }

            if key.code == key::W { step(&mut state, 0); }
            if key.code == key::E { step(&mut state, 1); }
            if key.code == key::D { step(&mut state, 2); }
            if key.code == key::S { step(&mut state, 3); }
            if key.code == key::A { step(&mut state, 4); }
            if key.code == key::Q { step(&mut state, 5); }

            if key.code == key::F12 {
                app.screenshot("/tmp/shot.png");
            }
        }

        app.flush();
    }

    fn step(state: &mut Roamy, dir: uint) {
        // Steer to the sides if bump.
        if !state.step(&DIRECTIONS6[dir]) {
            if !state.step(&DIRECTIONS6[(dir + 1) % 6]) {
                state.step(&DIRECTIONS6[(dir + 5) % 6]);
            }
        }
    }
}

