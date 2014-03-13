#[feature(globs)];
extern crate cgmath;
extern crate glutil;
extern crate color;
extern crate calx;
extern crate stb;
extern crate collections;
extern crate num;

use glutil::glrenderer::GlRenderer;
use calx::key;
use calx::tile::Tile;
use calx::renderer::Renderer;
use calx::app::App;
use cgmath::vector::{Vec2};
use stb::image::Image;
use game::Game;
use area::DIRECTIONS6;

pub mod fov;
pub mod area;
pub mod areaview;
pub mod dijkstra;
pub mod game;
pub mod mapgen;
pub mod mob;
pub mod transform;
pub mod sprite;

static TILE_DATA: &'static [u8] = include!("../../gen/tile_data.rs");

pub fn main() {
    let mut app : App<GlRenderer> = App::new(640, 360, "Morlock Hunter");
    let tiles = Image::load_from_memory(TILE_DATA, 1).unwrap();
    let tiles = Tile::new_alpha_set(
        &Vec2::new(32, 32),
        &Vec2::new(tiles.width as int, tiles.height as int),
        tiles.pixels,
        &Vec2::new(-16, -16));
    for i in range(0, 64) {
        app.r.add_tile(~tiles[i].clone());
    }

    let mut game = Game::new();

    while app.r.alive {
        game.draw(&mut app);

        loop {
            match app.r.pop_key() {
                Some(key) => {
                    if key.code == key::ESC {
                        return;
                    }

                    if key.code == key::SPACE {
                        game.stop = !game.stop;
                    }

                    if key.code == key::W { step(&mut game, 0); }
                    if key.code == key::E { step(&mut game, 1); }
                    if key.code == key::D { step(&mut game, 2); }
                    if key.code == key::S { step(&mut game, 3); }
                    if key.code == key::A { step(&mut game, 4); }
                    if key.code == key::Q { step(&mut game, 5); }

                    if key.code == key::F12 {
                        app.r.screenshot("/tmp/shot.png");
                    }
                },
                None => { break; }
            }
        }

        app.r.flush();
    }

    fn step(game: &mut Game, dir: uint) {
        // Steer to the sides if bump.
        if !game.step(&DIRECTIONS6[dir]) {
            if !game.step(&DIRECTIONS6[(dir + 1) % 6]) {
                game.step(&DIRECTIONS6[(dir + 5) % 6]);
            }
        }
    }
}
