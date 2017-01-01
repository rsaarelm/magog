use std::fs::File;
use std::io::prelude::*;
use std::collections::HashSet;
use std::num::Wrapping;
use euclid::{Point2D, Rect, Size2D};
use scancode::Scancode;
use vitral::Context;
use calx_grid::{Dir6, Prefab};
use world::{self, Form, Location, Mutate, Portal, Query, Terraform, World, Terrain};
use world::errors::*;
use display;
use vitral::{self, ButtonAction, Align, FracPoint2D};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum PaintMode {
    Terrain,
    Entity,
    Portal,
}

/// Top-level application state for gameplay.
pub struct View {
    pub world: World,

    fore_terrain: Terrain,
    back_terrain: Terrain,
    entity: String,
    mode: PaintMode,

    /// Camera and second camera (for portaling)
    camera: (Location, Location),
    /// Do the two cameras move together?
    camera_lock: bool,

    console: display::Console,
    console_is_large: bool,
}

impl View {
    pub fn new(world: World) -> View {
        View {
            world: world,
            fore_terrain: Terrain::Rock,
            back_terrain: Terrain::Ground,
            entity: "player".to_string(),
            mode: PaintMode::Terrain,
            camera: (Location::new(0, 0, 0), Location::new(0, 8, 0)),
            camera_lock: false,
            console: display::Console::new(),
            console_is_large: false,
        }
    }

    fn spawn_at(&mut self, loc: Location, form: Option<&str>) {
        for e in self.world.entities_at(loc) {
            self.world.remove_entity(e);
        }

        if let Some(name) = form {
            let form = Form::named(name).expect(&format!("Form '{}' not found!", name));
            self.world.spawn(&form.loadout, loc);
        }
    }

    pub fn draw(&mut self, context: &mut display::Backend, screen_area: &Rect<f32>) {
        let camera_loc = self.camera.0;
        let mut view = display::WorldView::new(camera_loc, *screen_area);
        view.show_cursor = true;
        view.highlight_offscreen_tiles = true;

        {
            let mut context = context.bound_clipped_r(*screen_area);

            view.draw(&self.world, &mut context);

            if let Some(loc) = view.cursor_loc {
                match self.mode {
                    PaintMode::Terrain => {
                        if context.is_mouse_pressed(vitral::MouseButton::Left) {
                            self.world.set_terrain(loc, self.fore_terrain);
                        }

                        if context.is_mouse_pressed(vitral::MouseButton::Right) {
                            self.world.set_terrain(loc, self.back_terrain);
                        }
                    }

                    PaintMode::Entity => {
                        if context.is_mouse_pressed(vitral::MouseButton::Left) {
                            // XXX: Cloning to evade borrow checker...
                            let e = self.entity.clone();
                            self.spawn_at(loc, Some(&e[..]));
                        }

                        if context.is_mouse_pressed(vitral::MouseButton::Right) {
                            self.spawn_at(loc, None);
                        }
                    }

                    PaintMode::Portal => {
                        let (a, b) = self.camera;
                        if a != b && context.is_mouse_pressed(vitral::MouseButton::Left) {
                            self.world.set_portal(loc, Portal::new(a, b));
                        }
                        if context.is_mouse_pressed(vitral::MouseButton::Right) {
                            self.world.remove_portal(loc);
                        }
                    }
                }
            }
        }

        {
            let mut context = context.bound(0, 360, 640, 120);

            match context.bound(10, 10, 100, 10).button(&format!("left: {:?}", self.fore_terrain)) {
                ButtonAction::LeftClicked => {
                    self.mode = PaintMode::Terrain;
                    self.fore_terrain = prev_terrain(self.fore_terrain);
                }
                ButtonAction::RightClicked => {
                    self.mode = PaintMode::Terrain;
                    self.fore_terrain = next_terrain(self.fore_terrain);
                }
                _ => {}
            };

            match context.bound(112, 10, 100, 10).button(&format!("right: {:?}", self.back_terrain)) {
                ButtonAction::LeftClicked => {
                    self.mode = PaintMode::Terrain;
                    self.back_terrain = prev_terrain(self.back_terrain);
                }
                ButtonAction::RightClicked => {
                    self.mode = PaintMode::Terrain;
                    self.back_terrain = next_terrain(self.back_terrain);
                }
                _ => {}
            };

            match context.bound(10, 22, 100, 10).button(&format!("entity: {}", self.entity)) {
                ButtonAction::LeftClicked => {
                    self.mode = PaintMode::Entity;

                    let names: Vec<&str> = world::FORMS.iter().filter_map(|x| x.name()).collect();
                    let idx = names.iter()
                                   .position(|x| *x == &self.entity[..])
                                   .expect(&format!("Invalid current entity '{}'", self.entity));

                    self.entity = names[(idx + (names.len() - 1)) % names.len()].to_string();
                }
                ButtonAction::RightClicked => {
                    self.mode = PaintMode::Entity;

                    let names: Vec<&str> = world::FORMS.iter().filter_map(|x| x.name()).collect();
                    let idx = names.iter()
                                   .position(|x| *x == &self.entity[..])
                                   .expect(&format!("Invalid current entity '{}'", self.entity));

                    self.entity = names[(idx + 1) % names.len()].to_string();
                }
                _ => {}
            };


            let mut pos = FracPoint2D::new(1.0, 0.05);
            for loc in view.cursor_loc.iter() {
                pos = context.draw_text(pos, Align::Right, [1.0, 1.0, 1.0, 1.0], &format!("{:?}", loc));
            }

        }

        if self.console_is_large {
            let mut console_area = *screen_area;
            console_area.size.height /= 2.0;
            self.console.draw_large(context, &console_area);
        } else {
            self.console.draw_small(context, screen_area);
        }

        if let Some(scancode) = context.poll_key().and_then(|k| Scancode::new(k.scancode)) {
            if self.console_is_large {
                self.console_input(context, scancode);
            } else {
                self.editor_input(context, scancode);
            }
        }
    }

    command_parser!{
        fn load(&mut self, path: String);
        fn save(&mut self, path: String);
    }

    fn load(&mut self, path: String) {
        fn loader(path: String) -> Result<Prefab<(Terrain, Vec<String>)>> {
            let mut file = File::open(path)?;
            world::load_prefab(&mut file)
        }

        let prefab = match loader(path) {
            Ok(prefab) => prefab,
            Err(e) => {
                let _ = writeln!(&mut self.console, "Load error: {}", e);
                return;
            }
        };

        // Apply map.
        self.world = World::new(1);
        // Prefabs do not contain positioning data. The standard fullscreen prefab which includes a
        // one-cell wide offscreen boundary goes in a bounding box with origin at (-21, -22).
        self.world.deploy_prefab(Location::new(-21, -22, 0), &prefab);

        // We don't want mapedit to have an active player entity.
        self.world.set_player(None);
    }

    fn save(&mut self, path: String) {
        let mut locs = HashSet::new();
        for x in world::onscreen_locations().iter() {
            locs.insert(Location::origin() + *x);
            // Get the immediate border around the actual screen cells, these will also affect the
            // visual look of the map.
            for d in Dir6::iter() {
                locs.insert(Location::origin() + *x + d.to_v2());
            }
        }

        let prefab = self.world.extract_prefab(locs.iter().map(|&x| x));

        match File::create(&path) {
            Ok(mut file) => {
                if let Err(e) = world::save_prefab(&mut file, &prefab) {
                    let _ = writeln!(&mut self.console, "Save failed: {}", e);
                } else {
                    let _ = writeln!(&mut self.console, "Saved '{}'", path);
                }
            }
            Err(e) => {
                let _ = writeln!(&mut self.console, "Couldn't open file {}: {:?}", path, e);
            }
        }
    }

    fn console_input(&mut self, context: &mut display::Backend, scancode: Scancode) {
        use scancode::Scancode::*;
        match scancode {
            Tab => {
                self.console_is_large = !self.console_is_large;
            }
            Enter | PadEnter => {
                let input = self.console.get_input();
                let _ = writeln!(&mut self.console, "{}", input);
                if let Err(e) = self.parse(&input) {
                    let _ = writeln!(&mut self.console, "{}", e);
                }
            }
            F12 => context.save_screenshot("mapedit"),
            _ => {}
        }
    }

    fn editor_input(&mut self, context: &mut display::Backend, scancode: Scancode) {
        use scancode::Scancode::*;
        match scancode {
            Q => self.move_camera(Point2D::new(-1, 0), 0),
            W => self.move_camera(Point2D::new(-1, -1), 0),
            E => self.move_camera(Point2D::new(0, -1), 0),
            A => self.move_camera(Point2D::new(0, 1), 0),
            S => self.move_camera(Point2D::new(1, 1), 0),
            D => self.move_camera(Point2D::new(1, 0), 0),
            F1 => self.switch_camera(),
            F12 => context.save_screenshot("mapedit"),
            Tab => self.console_is_large = !self.console_is_large,
            RightBracket => self.move_camera(Point2D::new(0, 0), 1),
            LeftBracket => self.move_camera(Point2D::new(0, 0), -1),
            _ => {}
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

fn next_terrain(t: Terrain) -> Terrain {
    let mut i = Terrain::iter().cycle().filter(|t| !t.is_irregular());
    loop {
        if i.next() == Some(&t) {
            return *i.next().unwrap();
        }
    }
}

fn prev_terrain(t: Terrain) -> Terrain {
    let mut i = Terrain::iter().rev().cycle().filter(|t| !t.is_irregular());
    loop {
        if i.next() == Some(&t) {
            return *i.next().unwrap();
        }
    }
}
