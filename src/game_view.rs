use euclid::{Point2D, Rect};
use calx_resource::Resource;
use scancode::Scancode;
use world::{Location, Portal, World};
use sprite::Sprite;
use vitral;
use backend;
use view;
use render;

enum PaintMode {
    Terrain(u8, u8),
    Portal,
}

/// Top-level application state for gameplay.
pub struct GameView {
    pub world: World,
    mode: PaintMode,
    /// Camera and second camera (for portaling)
    camera: (Location, Location),
    /// Do the two cameras move together?
    camera_lock: bool,
}

impl GameView {
    pub fn new(world: World) -> GameView {
        GameView {
            world: world,
            mode: PaintMode::Terrain(7, 3),
            camera: (Location::new(0, 0, 0), Location::new(0, 0, 0)),
            camera_lock: false,
        }
    }

    pub fn draw(&mut self, context: &mut backend::Context, screen_area: &Rect<f32>) {
        // TODO: Camera logic
        let camera_loc = self.camera.0;

        let center = screen_area.origin + screen_area.size / 2.0;

        // Chart area, center in origin, inflated by tile width in every direction to get the cells
        // partially on screen included.
        let bounds = screen_area.translate(&-(center + screen_area.origin))
                                .inflate(view::PIXEL_UNIT * 2.0, view::PIXEL_UNIT * 2.0);

        context.ui.set_clip_rect(Some(*screen_area));

        let chart = view::screen_fov(&self.world, camera_loc, bounds);

        let mut sprites = Vec::new();

        let cursor_pos = view::view_to_chart(context.ui.mouse_pos() - center);

        for (&chart_pos, origins) in &chart {
            assert!(!origins.is_empty());

            let loc = origins[0] + chart_pos;

            let screen_pos = view::chart_to_view(chart_pos) + center;

            // TODO: Set up dynamic lighting, shade sprites based on angle and local light.
            render::draw_terrain_sprites(&self.world, loc, |layer, _angle, brush, frame_idx| {
                sprites.push(Sprite {
                    layer: layer,
                    offset: [screen_pos.x as i32, screen_pos.y as i32],
                    brush: brush.clone(),
                    frame_idx: frame_idx,
                })
            });
            if self.world.portals.get(&loc).is_some() {
                sprites.push(Sprite {
                    layer: render::Layer::Decal,
                    offset: [screen_pos.x as i32, screen_pos.y as i32],
                    brush: Resource::new("portal".to_string()).unwrap(),
                    frame_idx: 0,
                });
            }
        }

        if let Some(origins) = chart.get(&cursor_pos) {
            let screen_pos = view::chart_to_view(cursor_pos) + center -
                             Point2D::new(view::PIXEL_UNIT, view::PIXEL_UNIT);
            let loc = origins[0] + cursor_pos;

            sprites.push(Sprite {
                layer: render::Layer::Decal,
                offset: [screen_pos.x as i32, screen_pos.y as i32],
                brush: Resource::new("cursor".to_string()).unwrap(),
                frame_idx: 0,
            });
            sprites.push(Sprite {
                layer: render::Layer::Effect,
                offset: [screen_pos.x as i32, screen_pos.y as i32],
                brush: Resource::new("cursor_top".to_string()).unwrap(),
                frame_idx: 0,
            });

            match self.mode {
                PaintMode::Terrain(draw, erase) => {
                    if context.ui.is_mouse_pressed(vitral::MouseButton::Left) {
                        self.world.terrain.set(loc, draw);
                    }

                    if context.ui.is_mouse_pressed(vitral::MouseButton::Right) {
                        self.world.terrain.set(loc, erase);
                    }
                }

                PaintMode::Portal => {
                    let (a, b) = self.camera;
                    if a != b && context.ui.is_mouse_pressed(vitral::MouseButton::Left) {
                        self.world.portals.insert(loc, Portal::new(a, b));
                    }
                    if context.ui.is_mouse_pressed(vitral::MouseButton::Right) {
                        self.world.portals.remove(&loc);
                    }
                }
            }

        }

        sprites.sort();

        for i in &sprites {
            i.draw(&mut context.ui)
        }

        if context.ui.button("draw void") {
            self.mode = PaintMode::Terrain(0, 3);
        }

        if context.ui.button("draw gate") {
            self.mode = PaintMode::Terrain(1, 3);
        }

        if context.ui.button("draw wall") {
            self.mode = PaintMode::Terrain(6, 3);
        }

        if context.ui.button("draw rock") {
            self.mode = PaintMode::Terrain(7, 3);
        }

        if context.ui.button("PORTALS!") {
            self.mode = PaintMode::Portal;
        }

        context.ui.set_clip_rect(None);

        if let Some(scancode) = context.backend.poll_key().and_then(|k| Scancode::new(k.scancode)) {
            use scancode::Scancode::*;
            match scancode {
                W => self.move_camera(Point2D::new(-1, -1)),
                A => self.move_camera(Point2D::new(-1, 1)),
                S => self.move_camera(Point2D::new(1, 1)),
                D => self.move_camera(Point2D::new(1, -1)),
                Tab => self.switch_camera(),
                _ => {}
            }
        }
    }

    fn move_camera(&mut self, delta: Point2D<i32>) {
        let second_delta = if self.camera_lock {
            delta
        } else {
            Point2D::new(0, 0)
        };

        let (a, b) = self.camera;
        self.camera = (a + delta, b + second_delta);
    }

    fn switch_camera(&mut self) {
        let (a, b) = self.camera;
        self.camera = (b, a);
    }
}
