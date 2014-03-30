#![feature(globs)]

extern crate cgmath;
extern crate glutil;
extern crate color;
extern crate calx;
extern crate time;
extern crate rand;
extern crate world;

use glutil::glrenderer::GlRenderer;
use calx::key;
use calx::renderer::Renderer;
use calx::app::App;
use calx::timing::Ticker;
use cgmath::vector::{Vec2};
use world::area::DIRECTIONS6;
use world::transform::Transform;
use world::areaview;
use game::Game;

pub mod game;

static SMART_MOVE_6: &'static [&'static [Vec2<int>]] = &[
    &[DIRECTIONS6[0], DIRECTIONS6[5], DIRECTIONS6[1]],
    &[DIRECTIONS6[1], DIRECTIONS6[0], DIRECTIONS6[2]],
    &[DIRECTIONS6[2], DIRECTIONS6[1], DIRECTIONS6[3]],
    &[DIRECTIONS6[3], DIRECTIONS6[2], DIRECTIONS6[4]],
    &[DIRECTIONS6[4], DIRECTIONS6[3], DIRECTIONS6[5]],
    &[DIRECTIONS6[5], DIRECTIONS6[4], DIRECTIONS6[0]],

    // Sideways move left, right, even column.
    &[DIRECTIONS6[5], DIRECTIONS6[4]],
    &[DIRECTIONS6[1], DIRECTIONS6[2]],

    // Sideways move left, right, odd column.
    &[DIRECTIONS6[4], DIRECTIONS6[5]],
    &[DIRECTIONS6[2], DIRECTIONS6[1]],
];

pub fn main() {
    let mut app : App<GlRenderer> = App::new(640, 360, "Demogame");
    areaview::init_tiles(&mut app);

    let mut game = Game::new();

    let mut standalone_anim = Ticker::new(0.2f64);

    while app.r.alive {
        game.draw(&mut app);

        let xf = Transform::new(game.pos);
        let mouse = app.r.get_mouse();
        let cursor_chart_loc = xf.to_chart(&mouse.pos);
        if game.has_player() {
            let _vec = cursor_chart_loc - game.pos;
            // TODO: Convert mouse clicks to player input. This needs a
            // slowdown feature to make it work like the keyboard repeat thing,
            // otherwise pressing the mouse button will make the inputs repeat
            // at top speed and make the game unplayable.
        }


        loop {
            // Player's gone, assume we're running an attract mode or something.
            if !game.has_player() { break; }
            if !game.player().is_alive() {
                if standalone_anim.get() {
                    game.update();
                }
            }

            let mut column;

            {
                let player = game.player();
                // For the hacked sideways move.
                column = player.loc.p().x - player.loc.p().y;
            }

            match app.r.pop_key() {
                Some(key) => {
                    if game.player().is_alive() {
                        match key.code {

                            key::Q | key::HOME => { game.smart_move(SMART_MOVE_6[5]); },
                            key::W | key::UP => { game.smart_move(SMART_MOVE_6[0]); },
                            key::E | key::PAGEUP => { game.smart_move(SMART_MOVE_6[1]); },
                            key::A | key::END => { game.smart_move(SMART_MOVE_6[4]); },
                            key::S | key::DOWN => { game.smart_move(SMART_MOVE_6[3]); },
                            key::D | key::PAGEDOWN => { game.smart_move(SMART_MOVE_6[2]); },

                            key::LEFT => { game.smart_move(SMART_MOVE_6[ if column % 2 == 0 { 6 } else { 8 }]); },
                            key::RIGHT => { game.smart_move(SMART_MOVE_6[ if column % 2 == 0 { 7 } else { 9 }]); },
                            key::SPACE => { game.pass(); },
                            _ => (),
                        }
                    }
                    match key.code {
                        key::ESC => { return; },
                        key::F12 => { app.r.screenshot("/tmp/shot.png"); },
                        _ => (),
                    }
                },
                None => { break; }
            }
        }

        app.r.flush();
    }
}
