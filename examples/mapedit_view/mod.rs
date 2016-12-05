use std::collections::HashMap;
use std::num::Wrapping;
use euclid::{Point2D, Rect};
use scancode::Scancode;
use world::{Location, Portal, Terraform, World};
use display;
use vitral;

enum PaintMode {
    Terrain(u8, u8),
    Portal,
}

/// Top-level application state for gameplay.
pub struct View {
    pub world: World,
    mode: PaintMode,
    /// Camera and second camera (for portaling)
    camera: (Location, Location),
    /// Do the two cameras move together?
    camera_lock: bool,
}

impl View {
    pub fn new(world: World) -> View {
        View {
            world: world,
            mode: PaintMode::Terrain(7, 2),
            camera: (Location::new(0, 0, 0), Location::new(0, 8, 0)),
            camera_lock: false,
        }
    }

    pub fn draw(&mut self, context: &mut display::Context, screen_area: &Rect<f32>) {
        let camera_loc = self.camera.0;

        let mut view = display::WorldView::new(camera_loc, *screen_area);
        view.show_cursor = true;
        view.highlight_offscreen_tiles = true;
        view.draw(&self.world, context);

        if let Some(loc) = view.cursor_loc {
            match self.mode {
                PaintMode::Terrain(draw, erase) => {
                    if context.ui.is_mouse_pressed(vitral::MouseButton::Left) {
                        self.world.set_terrain(loc, draw);
                    }

                    if context.ui.is_mouse_pressed(vitral::MouseButton::Right) {
                        self.world.set_terrain(loc, erase);
                    }
                }

                PaintMode::Portal => {
                    let (a, b) = self.camera;
                    if a != b && context.ui.is_mouse_pressed(vitral::MouseButton::Left) {
                        self.world.set_portal(loc, Portal::new(a, b));
                    }
                    if context.ui.is_mouse_pressed(vitral::MouseButton::Right) {
                        self.world.remove_portal(loc);
                    }
                }
            }

        }

        if context.ui.button("draw void") {
            self.mode = PaintMode::Terrain(0, 2);
        }

        if context.ui.button("draw gate") {
            self.mode = PaintMode::Terrain(1, 2);
        }

        if context.ui.button("draw wall") {
            self.mode = PaintMode::Terrain(6, 2);
        }

        if context.ui.button("draw rock") {
            self.mode = PaintMode::Terrain(7, 2);
        }

        if context.ui.button("PORTALS!") {
            self.mode = PaintMode::Portal;
        }

        for (y, loc) in view.cursor_loc.iter().enumerate() {
            let font = context.ui.default_font();
            context.ui.draw_text(&*font,
                                 Point2D::new(400.0, y as f32 * 20.0 + 20.0),
                                 [1.0, 1.0, 1.0, 1.0],
                                 &format!("{:?}", loc));
        }

        context.ui.set_clip_rect(None);

        if let Some(scancode) = context.backend.poll_key().and_then(|k| Scancode::new(k.scancode)) {
            use scancode::Scancode::*;
            match scancode {
                Q => self.move_camera(Point2D::new(-1, 0), 0),
                W => self.move_camera(Point2D::new(-1, -1), 0),
                E => self.move_camera(Point2D::new(0, -1), 0),
                A => self.move_camera(Point2D::new(0, 1), 0),
                S => self.move_camera(Point2D::new(1, 1), 0),
                D => self.move_camera(Point2D::new(1, 0), 0),
                Tab => self.switch_camera(),
                RightBracket => self.move_camera(Point2D::new(0, 0), 1),
                LeftBracket => self.move_camera(Point2D::new(0, 0), -1),
                _ => {}
            }
        }
    }

    fn move_camera(&mut self, delta: Point2D<i32>, dz: i8) {
        let second_delta = if self.camera_lock { delta } else { Point2D::new(0, 0) };

        let (a, b) = self.camera;
        self.camera = (a + delta, b + second_delta);

        let z0 = Wrapping(self.camera.0.z) + Wrapping(dz);
        let z1 = Wrapping(self.camera.1.z) + Wrapping(if self.camera_lock { dz } else { 0 });

        self.camera.0.z = z0.0;
        self.camera.1.z = z1.0;
    }

    fn switch_camera(&mut self) {
        let (a, b) = self.camera;
        self.camera = (b, a);
    }
}

/// Type for maps saved into disk.
#[derive(Debug, RustcEncodable, RustcDecodable)]
struct MapSave {
    pub map: String,
    pub legend: HashMap<char, LegendItem>,
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
struct LegendItem {
    /// Terrain
    pub t: String,
    /// Entities
    pub e: Vec<String>,
}
