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
use calx::backend::{CanvasBuilder, Canvas, CanvasUtil, Event, MouseButton, Key};
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

pub trait Drawable {
    fn draw(&self, ctx: &mut Canvas, offset: V2<f32>, z: f32);
}

pub struct SprDrawable {
    pub spr: Spr,
    pub fore: Rgba,
    pub back: Rgba,
}

impl SprDrawable {
    pub fn new<A: Into<Rgba>, B: Into<Rgba>>(spr: Spr, fore: A, back: B) -> SprDrawable {
        SprDrawable {
            spr: spr,
            fore: fore.into(),
            back: back.into(),
        }
    }
}

impl Drawable for SprDrawable {
    fn draw(&self, ctx: &mut Canvas, offset: V2<f32>, z: f32) {
        ctx.draw_image(self.spr.get(), offset, z, self.fore, self.back);
    }
}

pub struct Sprite {
    pub drawable: Box<Drawable>,
    pub pos: V2<f32>,

    sort_key: (i8, i32),
}

impl Sprite {
    pub fn new(drawable: Box<Drawable>, pos: V2<f32>, layer: i8) -> Sprite {
        // Fix some numerical inaccuracy noise.
        let pos = pos.map(|x| x.round());
        Sprite {
            drawable: drawable,
            pos: pos,
            sort_key: (layer, -pos.1 as i32)
        }
    }

    pub fn new_spr<A: Into<Rgba>, B: Into<Rgba>>(
        spr: Spr, fore: A, back: B, pos: V2<f32>, layer: i8) -> Sprite {
        Sprite::new(Box::new(SprDrawable::new(spr, fore, back)), pos, layer)
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
        ret.push(Sprite::new_spr(Spr::EdgeNW + idx, color, color::BLACK, screen_pos, 0));
    }
    ret
}

#[derive(Copy, Clone)]
pub struct Effect {
    pub kind: Fx,
    pub life: u32,
    pub pos: V2<i32>,
}

impl Effect {
    pub fn new(kind: Fx, map_pos: V2<i32>) -> Effect {
        // TODO: Lifetime parametrization.
        Effect {
            kind: kind,
            life: 10,
            pos: map_pos,
        }
    }

    pub fn update(&mut self) { if self.life > 0 { self.life -= 1; } }
    pub fn is_alive(&self) -> bool { self.life > 0 }

    pub fn sprite(&self, proj: &Projection) -> Sprite {
        // XXX: Cloning self, inefficient.
        Sprite::new(Box::new(*self), proj.project(self.pos.map(|x| (x as f32))), -4)
    }
}

impl Drawable for Effect {
    fn draw(&self, ctx: &mut Canvas, offset: V2<f32>, z: f32) {
        match self.kind {
            Fx::PathOk => {
                ctx.draw_rect(&Rect(offset + V2(-8.0, 0.0), V2(17.0, 16.0)), z, color::LIME);
            }
            Fx::PathBlocked => {
                ctx.draw_line(2.0, offset + V2(-8.0, 0.0), offset + V2(8.0, 16.0), z, color::RED);
                ctx.draw_line(2.0, offset + V2(-8.0, 16.0), offset + V2(8.0, 0.0), z, color::RED);
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Fx {
    PathOk,
    PathBlocked,
}

#[derive(Eq, PartialEq)]
pub enum GameMode {
    RealTime,
    Paused,
    Rogue(Entity),
}

pub struct GameState {
    world: World,

    selected: HashSet<Entity>,
    mode: GameMode,

    screen_offset: V2<f32>,
    scroll_delta: V2<f32>,
    pub proj: Projection,
    mouse_cell: V2<i32>,
    drag_rect: Option<Rect<f32>>,

    effects: Vec<Box<Effect>>,
}

impl GameState {
    pub fn new(world: World) -> GameState {
        let screen_offset = V2(320.0f32, 0.0f32);
        GameState {
            world: world,
            selected: HashSet::new(),
            mode: GameMode::RealTime,
            screen_offset: screen_offset,
            scroll_delta: V2(0.0, 0.0),
            proj: Projection::new(V2(16.0, 8.0), V2(-16.0, 8.0)).unwrap()
                .view_offset(screen_offset),
            mouse_cell: V2(-1, -1),
            drag_rect: None,
            effects: Vec::new(),
        }
    }

    pub fn render(&self, ctx: &mut Canvas) {
        let screen_rect = Rect(V2(-32.0f32, -32.0f32), V2(640.0f32 + 64.0, 360.0f32 + 64.0));

        let mut sprites = Vec::new();

        for pt in self.proj.inv_project_rectangle(&screen_rect).iter() {
            let pos = self.proj.project(pt);
            Kernel::new(|p| self.world.terrain_at(p), pt.map(|x| x as i32)).render(
                |layer, spr, fore, back| {
                    sprites.push(Sprite::new_spr(spr, fore, back, pos, layer));
                });
        }

        for &e in self.world.ecs.iter() {
            if let Some(spr) = cmd::sprite(&self.world, e, &self.proj) {
                // Highlight reticle on focused unit.
                if self.selected.contains(&e) {
                    for s in unit_focus_sprites(spr.pos).into_iter() {
                        sprites.push(s);
                    }
                }
                sprites.push(spr);
            }
        }

        for f in self.effects.iter() {
            sprites.push(f.sprite(&self.proj));
        }

        sprites.sort_by(|a, b| a.cmp(&b));
        for spr in sprites.iter() {
            spr.drawable.draw(ctx, spr.pos, 0.5);
        }

        if let Some(rect) = self.drag_rect {
            ctx.draw_rect(&rect, 0.2, color::CYAN);
        }
    }

    pub fn update(&mut self) {
        if self.mode == GameMode::RealTime {
            self.world.update_active();
        } else {
            self.world.update_standby();
        }

        match self.mode {
            GameMode::Rogue(rogue) => {
                let proj = Projection::new(V2(16.0, 8.0), V2(-16.0, 8.0)).unwrap();
                let spr = cmd::sprite(&self.world, rogue, &proj).unwrap();
                self.screen_offset = -spr.pos + V2(320.0, 180.0);
            }
            _ => { self.screen_offset = self.screen_offset - self.scroll_delta; }
        }

        self.proj = Projection::new(V2(16.0, 8.0), V2(-16.0, 8.0)).unwrap()
            .view_offset(self.screen_offset);

        for e in self.effects.iter_mut() {
            e.update();
        }

        self.effects.retain(|e| e.is_alive());
    }

    fn go_rogue(&mut self) -> Option<Entity> {
        if let GameMode::Rogue(rogue) = self.mode { return Some(rogue); }

        if let Some(&rogue) = self.selected.iter().next() {
            self.selected.clear();
            self.mode = GameMode::Rogue(rogue);
            Some(rogue)
        } else {
            None
        }
    }

    fn toggle_pause(&mut self) {
        match self.mode {
            GameMode::Rogue(_) | GameMode::Paused => self.mode = GameMode::RealTime,
            GameMode::RealTime => self.mode = GameMode::Paused,
        }
    }

    fn rogue_step(&mut self, dir: Dir6) {
        if let Some(rogue) = self.go_rogue() {
            while !cmd::ready_to_act(&self.world, rogue) { self.world.update_active(); }
            self.smart_move(rogue, dir);
        }
    }

    fn smart_move(&mut self, e: Entity, dir: Dir6) {
        let pos = self.world.ecs.pos[e];
        let target_pos = pos + dir.to_v2();

        if let Some(_enemy) = cmd::mob_at(&self.world, target_pos).map_or(
            None,
            |x| if !cmd::is_player(&self.world, x) { Some(x) } else { None }) {
            cmd::melee(&mut self.world, e, dir);
        } else {
            // TODO: Wall-hugging.
            cmd::step(&mut self.world, e, dir);
        }
    }

    fn add_effect(&mut self, effect: Box<Effect>) {
        self.effects.push(effect);
    }

    pub fn process_event(&mut self, evt: Event) {
        let scroll_speed = 8f32;
        match evt {
            Event::KeyPress(k) => {
                match k {
                    Key::J => { self.scroll_delta.0 = -1.0 * scroll_speed; }
                    Key::L => { self.scroll_delta.0 =  1.0 * scroll_speed; }
                    Key::I => { self.scroll_delta.1 = -1.0 * scroll_speed; }
                    Key::K => { self.scroll_delta.1 =  1.0 * scroll_speed; }
                    Key::Space => { self.toggle_pause(); }

                    Key::W => { self.rogue_step(Dir6::North); }
                    Key::E => { self.rogue_step(Dir6::NorthEast); }
                    Key::D => { self.rogue_step(Dir6::SouthEast); }
                    Key::S => { self.rogue_step(Dir6::South); }
                    Key::A => { self.rogue_step(Dir6::SouthWest); }
                    Key::Q => { self.rogue_step(Dir6::NorthWest); }
                    _ => {}
                }
            }

            Event::KeyRelease(k) => {
                match k {
                    Key::J => { self.scroll_delta.0 = 0.0; }
                    Key::L => { self.scroll_delta.0 = 0.0; }
                    Key::I => { self.scroll_delta.1 = 0.0; }
                    Key::K => { self.scroll_delta.1 = 0.0; }
                    _ => {}
                }
            }

            Event::MouseDrag(MouseButton::Left, p1, p2) => {
                self.drag_rect = Some(Rect::from_points(p1, p2))
            }

            Event::MouseMove(pos) => {
                self.mouse_cell = self.proj.inv_project(pos).map(|x| x.floor() as i32);
            }

            Event::MouseClick(MouseButton::Left) => {
                self.selected.clear();
                if let Some(p) = cmd::mob_at(&self.world, self.mouse_cell) {
                    self.selected.insert(p);
                }
            }

            Event::MouseClick(MouseButton::Right) => {
                let target = cmd::mob_at(&self.world, self.mouse_cell).map_or(
                   None,
                   |x| if !cmd::is_player(&self.world, x) { Some(x) } else { None });

                let mut path_found = None;
                for &unit in self.selected.iter() {
                    if cmd::is_player(&self.world, unit) {
                        let cmd_pathed = if let Some(enemy) = target {
                            cmd::attack(&mut self.world, unit, enemy)
                        } else {
                            cmd::move_to(&mut self.world, unit, self.mouse_cell)
                        };

                        path_found = path_found.map_or(
                            Some(cmd_pathed), |x| Some(x || cmd_pathed));
                    }
                }

                let pos = self.mouse_cell;
                match path_found {
                    Some(true) =>
                        self.add_effect(Box::new(Effect::new(Fx::PathOk, pos))),
                    Some(false) =>
                        self.add_effect(Box::new(Effect::new(Fx::PathBlocked, pos))),
                    _ => {}
                }
            }

            Event::MouseDragEnd(MouseButton::Left, p1, p2) => {
                let scale = (p2.0 - p1.0).abs().max((p2.1 - p1.1).abs());
                // Must have dragged some distance before we go from looking
                // at the single click to looking at the dragged rectangle.
                if scale > 12.0 {
                    self.selected.clear();
                    self.drag_rect = None;

                    let cell_rect = Rect(V2(-8f32, -8f32), V2(16f32, 8f32));
                    let select_rect = Rect::from_points(p1, p2);

                    for pt in self.proj.inv_project_rectangle(&select_rect).iter() {
                        if !select_rect.intersects(&(cell_rect + self.proj.project(pt))) { continue; }

                        let pos = pt.map(|x| x.floor() as i32);

                        match cmd::mob_at(&self.world, pos) {
                            Some(e) if cmd::is_player(&self.world, e) => {
                                self.selected.insert(e);
                            }
                            _ => {}
                        }
                    }
                }
            }

            _ => {}
        }
    }
}

fn main() {
    let mut world = World::new();
    let tmx = include_str!("../assets/hexworld.tmx");
    world.load(&tiled::parse(tmx.as_bytes()).unwrap());
    let mut game = GameState::new(world);

    let mut builder = CanvasBuilder::new()
        .set_size(640, 360)
        .set_frame_interval(0.033f64)
        ;
    Spr::init(&mut builder);
    let mut ctx = builder.build();

    loop {
        match ctx.next_event() {
            Event::RenderFrame => {
                game.update();
                game.render(&mut ctx);
            }

            Event::Quit => { return; }

            Event::KeyPress(Key::F12) => {
                ctx.save_screenshot(&"azag");
            }

            Event::KeyPress(Key::Escape) => {
                return;
            }

            e => game.process_event(e),
        }
    }
}
