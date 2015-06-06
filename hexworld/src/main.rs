/*! Hex map display demo */

extern crate num;
extern crate rustc_serialize;
extern crate image;
extern crate tiled;

#[macro_use] extern crate calx_ecs;
extern crate calx;

mod cmd;
mod render;
mod spr;
mod world;

use std::convert::{Into};
use std::collections::{HashSet};
use calx_ecs::{Entity};
use calx::backend::{CanvasBuilder, CanvasUtil, Event, MouseButton, Key};
use calx::{V2, Rect, Rgba, color, Dir6};
use calx::{Projection, Kernel, KernelTerrain};

use spr::Spr;
use render::RenderTerrain;
use world::World;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Terrain {
    Floor,
    Grass,
    Water,
    Tree,
    Wall,
    Door,
    Window,
    Magma,
    Rock,
    Void,
}

impl Terrain {
    pub fn new(id: u8) -> Terrain {
        // Tiled indexes start from 1.
        let id = id - 1;
        assert!(id <= Terrain::Rock as u8);
        unsafe {
            std::mem::transmute(id)
        }
    }

    pub fn can_walk(self) -> bool {
        use Terrain::*;
        match self {
            Floor | Grass | Door => true,
            _ => false,
        }
    }
}

impl KernelTerrain for Terrain {
    fn is_wall(&self) -> bool {
        use Terrain::*;
        match *self {
            Wall | Door | Window => true,
            _ => false
        }
    }

    fn is_block(&self) -> bool { *self == Terrain::Rock }
}

pub struct Sprite {
    pub spr: Spr,
    pub fore: Rgba,
    pub back: Rgba,
    pub pos: V2<f32>,

    sort_key: (i8, i32),
}

impl Sprite {
    pub fn new<A: Into<Rgba>, B: Into<Rgba>>(spr: Spr, pos: V2<f32>, layer: i8, fore: A, back: B) -> Sprite {
        // Fix some numerical inaccuracy noise.
        let pos = pos.map(|x| x.round());
        Sprite {
            spr: spr,
            fore: fore.into(),
            back: back.into(),
            pos: pos,
            sort_key: (layer, -pos.1 as i32)
        }
    }

    #[inline]
    pub fn cmp(&self, other: &Sprite) -> std::cmp::Ordering {
        // Cmp backwards, draw order is from large (far away) to small (close by) values.
        other.sort_key.cmp(&self.sort_key)
    }
}

fn unit_focus_sprites(screen_pos: V2<f32>) -> Vec<Sprite> {
    let mut ret = Vec::new();
    let color = color::LIME;
    for idx in 0..6 {
        ret.push(Sprite::new(Spr::EdgeNW + idx, screen_pos, 0, color, color::BLACK));
    }
    ret
}

fn main() {
    let scroll_speed = 8f32;
    let mut screen_offset = V2(320.0f32, 0.0f32);
    let mut scroll_delta = V2(0.0f32, 0.0f32);
    let mut mouse_cell = V2(-1, -1);
    let mut mouse_pos = V2(-1.0f32, -1.0f32);
    let mut rect1 = V2(-1.0f32, -1.0f32);

    let mut world = World::new();
    let mut active: HashSet<Entity> = HashSet::new();
    let mut rect_drag = None;

    let mut paused = false;

    let tmx = include_str!("../assets/hexworld.tmx");
    world.load(&tiled::parse(tmx.as_bytes()).unwrap());

    let screen_rect = Rect(V2(-32.0f32, -32.0f32), V2(640.0f32 + 64.0, 360.0f32 + 64.0));
    let mut builder = CanvasBuilder::new()
        .set_size(640, 360)
        .set_frame_interval(0.033f64)
        ;
    Spr::init(&mut builder);
    let mut ctx = builder.build();

    let mut proj = Projection::new(V2(16.0, 8.0), V2(-16.0, 8.0)).unwrap()
        .view_offset(screen_offset);
    loop {
        match ctx.next_event() {
            Event::RenderFrame => {
                screen_offset = screen_offset - scroll_delta;

                let mut sprites = Vec::new();

                proj = Projection::new(V2(16.0, 8.0), V2(-16.0, 8.0)).unwrap()
                    .view_offset(screen_offset);

                for pt in proj.inv_project_rectangle(&screen_rect).iter() {
                    let pos = proj.project(pt);
                    Kernel::new(|p| world.terrain_at(p), pt.map(|x| x as i32)).render(
                        |layer, spr, fore, back| {
                            sprites.push(Sprite::new(spr, pos, layer, fore, back));
                        });
                }

                for &e in world.ecs.iter() {
                    if let Some(spr) = cmd::sprite(&world, e, &proj) {
                        // Highlight reticle on focused unit.
                        if active.contains(&e) {
                            for s in unit_focus_sprites(spr.pos).into_iter() {
                                sprites.push(s);
                            }
                        }
                        sprites.push(spr);
                    }
                }

                sprites.sort_by(|a, b| a.cmp(&b));
                for spr in sprites.iter() {
                    ctx.draw_image(spr.spr.get(), spr.pos, 0.5, spr.fore, spr.back);
                }

                if rect_drag == Some(Drag::Moving) {
                    let rect = Rect::from_points(mouse_pos, rect1);
                    ctx.draw_rect(&rect, 0.2, color::CYAN);
                }

                if paused {
                    world.update_standby();
                } else {
                    world.update_active();
                }
            }

            Event::Quit => { return; }

            Event::KeyPressed(Key::Escape) => { return; }

            Event::KeyPressed(Key::F12) => {
                ctx.save_screenshot(&"azag");
            }

            Event::KeyPressed(k) => {
                match k {
                    Key::J => { scroll_delta.0 = -1.0 * scroll_speed; }
                    Key::L => { scroll_delta.0 =  1.0 * scroll_speed; }
                    Key::I => { scroll_delta.1 = -1.0 * scroll_speed; }
                    Key::K => { scroll_delta.1 =  1.0 * scroll_speed; }

                    // XXX: Okay, we need a state object here.
                    Key::W => {
                        if !active.is_empty() {
                            paused = true;
                            let rogue = activate_rogue(&mut active);
                            while !cmd::ready_to_act(&world, rogue) { world.update_active(); }
                            cmd::step(&mut world, rogue, Dir6::North);
                        }
                    }

                    Key::E => {
                        if !active.is_empty() {
                            paused = true;
                            let rogue = activate_rogue(&mut active);
                            while !cmd::ready_to_act(&world, rogue) { world.update_active(); }
                            cmd::step(&mut world, rogue, Dir6::NorthEast);
                        }
                    }
                    Key::D => {
                        if !active.is_empty() {
                            paused = true;
                            let rogue = activate_rogue(&mut active);
                            while !cmd::ready_to_act(&world, rogue) { world.update_active(); }
                            cmd::step(&mut world, rogue, Dir6::SouthEast);
                        }
                    }
                    Key::S => {
                        if !active.is_empty() {
                            paused = true;
                            let rogue = activate_rogue(&mut active);
                            while !cmd::ready_to_act(&world, rogue) { world.update_active(); }
                            cmd::step(&mut world, rogue, Dir6::South);
                        }
                    }
                    Key::A => {
                        if !active.is_empty() {
                            paused = true;
                            let rogue = activate_rogue(&mut active);
                            while !cmd::ready_to_act(&world, rogue) { world.update_active(); }
                            cmd::step(&mut world, rogue, Dir6::SouthWest);
                        }
                    }
                    Key::Q => {
                        if !active.is_empty() {
                            paused = true;
                            let rogue = activate_rogue(&mut active);
                            while !cmd::ready_to_act(&world, rogue) { world.update_active(); }
                            cmd::step(&mut world, rogue, Dir6::NorthWest);
                        }
                    }
                    Key::Space => { paused = !paused; }
                    _ => {}
                }
            }

            Event::KeyReleased(k) => {
                match k {
                    Key::J => { scroll_delta.0 = 0.0; }
                    Key::L => { scroll_delta.0 = 0.0; }
                    Key::I => { scroll_delta.1 = 0.0; }
                    Key::K => { scroll_delta.1 = 0.0; }
                    _ => {}
                }
            }
            Event::MouseMoved((x, y)) => {
                mouse_pos = V2(x, y).map(|x| x as f32);
                mouse_cell = proj.inv_project(mouse_pos).map(|x| x.floor() as i32);
                if rect_drag == Some(Drag::Click) {
                    let dist = rect1 - mouse_pos;
                    // Need to move past a threshold distance before we switch
                    // from click-selecting to rect-selecting.
                    if dist.dot(dist) > 16.0 {
                        rect_drag = Some(Drag::Moving);
                    }
                }
            }

            Event::MousePressed(MouseButton::Left) => {
                rect_drag = Some(Drag::Click);
                rect1 = mouse_pos;
            }

            Event::MouseReleased(MouseButton::Left) => {
                // Insert player units in select rectangle into selection.
                active.clear();
                match rect_drag {
                    Some(Drag::Moving) => {
                        let cell_rect = Rect(V2(-8f32, -8f32), V2(16f32, 8f32));
                        let select_rect = Rect::from_points(mouse_pos, rect1);

                        for pt in proj.inv_project_rectangle(&select_rect).iter() {
                            if !select_rect.intersects(&(cell_rect + proj.project(pt))) { continue; }

                            let pos = pt.map(|x| x.floor() as i32);

                            match cmd::mob_at(&world, pos) {
                                Some(e) if cmd::is_player(&world, e) => {
                                    active.insert(e);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {
                        if let Some(p) = cmd::mob_at(&world, mouse_cell) {
                            active.insert(p);
                        }
                    }
                }
                rect_drag = None;
            }

            Event::MousePressed(MouseButton::Right) => {
                for &p in active.iter() {
                    if cmd::is_player(&world, p) {
                        cmd::move_to(&mut world, p, mouse_cell);
                    }
                }
            }

            _ => {}
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Drag {
    Click,
    Moving,
}

fn activate_rogue(active: &mut HashSet<Entity>) -> Entity {
    let &player = active.iter().next().unwrap();
    active.clear();
    active.insert(player);
    player
}
