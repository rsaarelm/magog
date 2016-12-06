use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::error::Error;
use std::iter::FromIterator;
use std::fmt;
use std::collections::{HashMap, HashSet};
use std::num::Wrapping;
use euclid::{Point2D, Rect};
use rustc_serialize::Decodable;
use toml;
use scancode::Scancode;
use calx_grid::{Prefab, Dir6};
use world::{self, Location, Portal, TerrainQuery, Terraform, World};
use world::terrain;
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

    console: display::Console,
    console_is_large: bool,
}

impl View {
    pub fn new(world: World) -> View {
        View {
            world: world,
            mode: PaintMode::Terrain(7, 2),
            camera: (Location::new(0, 0, 0), Location::new(0, 8, 0)),
            camera_lock: false,
            console: display::Console::new(),
            console_is_large: false,
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

        if self.console_is_large {
            let mut console_area = *screen_area;
            console_area.size.height /= 2.0;
            self.console.draw_large(context, &console_area);
        } else {
            self.console.draw_small(context, screen_area);
        }

        if let Some(scancode) = context.backend.poll_key().and_then(|k| Scancode::new(k.scancode)) {
            if self.console_is_large {
                self.console_input(scancode);
            } else {
                self.editor_input(scancode);
            }
        }
    }

    command_parser!{
        fn load(&mut self, path: String);
        fn save(&mut self, path: String);
    }

    fn load(&mut self, path: String) {
        fn loader(path: String) -> Result<Prefab<terrain::Id>, Box<Error>> {
            let mut file = File::open(path)?;
            let mut s = String::new();
            file.read_to_string(&mut s)?;
            let mut parser = toml::Parser::new(&s);

            let mut decoder = toml::Decoder::new(
                toml::Value::Table(parser.parse().ok_or_else(|| format!("Parse errors: {:?}", parser.errors))?));
            let save = MapSave::decode(&mut decoder)?;

            // Turn map into prefab.
            let prefab: Prefab<char> = Prefab::from_text_hexmap(&save.map);
            let prefab: Prefab<terrain::Id> = prefab.map(|item| {
                // TODO: REALLY need error handling instead of panicing here, since we're possibly
                // dealing with input from the outside, but can't do error pattern in the map
                // closure. Fix by adding a pre-verify stage that checks that all map glyphs are
                // found in legend and that all legend items can be instantiated.
                let item = save.legend.get(&item).expect(&format!("Glyph '{}' not found in legend!", item));
                terrain::Id::from_str(&item.t).expect(&format!("Unknown terrain type '{}'!", item.t))
            });

            Ok(prefab)
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
        let offset = Point2D::new(-21, -22);

        for y in 0..prefab.dim().height {
            for x in 0..prefab.dim().width {
                let p = Point2D::new(x as i32, y as i32);
                if let Some(&t) = prefab.get(p) {
                    let loc = Location::origin() + p + offset;
                    self.world.set_terrain(loc, t as u8);
                }
            }
        }
    }

    fn save(&mut self, path: String) {
        let mut pts = HashSet::new();
        for x in world::onscreen_locations().iter() {
            pts.insert(*x);
            // Get the immediate border around the actual screen cells, these will also affect the
            // visual look of the map.
            for d in Dir6::iter() {
                pts.insert(*x + d.to_v2());
            }
        }

        let mut map = Vec::new();
        let mut legend = HashMap::new();

        for &p in pts.iter() {
            use world::terrain::Id::*;
            use world::terrain::Id;
            // TODO: Centralized place for this. Does not belong here...
            //
            // This will get more complex anyway once entity spawns are added to the mix, so hold
            // off doing the more engineered version for now. (Plan is to use more or less
            // conventional roguelike characters for plain terrain, then a sequence of alphabetical
            // chars diving off into unicode space if needed for each unique terrain + entity list
            // value.)
            //
            // TODO: Add unicode's confusables.txt official visual lookalike dataset,
            // eg at ftp://unicode.org/Public/security/8.0.0/confusables.txt
            // to filter the generated characters.
            //
            // The load code reads this stuff from the legend, doesn't need mapping logic.

            // NB: Since these are saved to TOML which uses the `'` single quote character to
            // separate literal strings, no legend item must use the single quote character.
            let id = self.world.terrain_id(Location::origin() + p);
            // XXX: Hacketyhack unsafe integer to enum.
            let id = unsafe { ::std::mem::transmute::<u8, Id>(id) };

            let c = match id {
                Empty => ' ',
                Gate => '^',
                Ground => '.',
                Grass => ',',
                Water => '~',
                Tree => '%',
                Wall => '#',
                Rock => '*',
                Corridor => '_',
                OpenDoor => '|',
                Door => '+',
            };

            legend.insert(c, LegendItem { t: format!("{:?}", id), e: Vec::new() });
            map.push((p, c));
        }

        let save = MapSave {
            map: format!("{}", Prefab::from_iter(map.into_iter()).hexmap_display()),
            legend: legend
        };

        match File::create(&path) {
            Ok(mut file) => {
                write!(file, "{}", save).expect("Write to file failed");
                let _ = writeln!(&mut self.console, "Saved '{}'", path);
            }
            Err(e) => { let _ = writeln!(&mut self.console, "Couldn't open file {}: {:?}", path, e); }
        }
    }

    fn console_input(&mut self, scancode: Scancode) {
        use scancode::Scancode::*;
        match scancode {
            Tab => { self.console_is_large = !self.console_is_large; }
            Enter | PadEnter => {
                let input = self.console.get_input();
                let _ = writeln!(&mut self.console, "{}", input);
                if let Err(e) = self.parse(&input) {
                    let _ = writeln!(&mut self.console, "{}", e);
                }
            }
            _ => {}
        }
    }

    fn editor_input(&mut self, scancode: Scancode) {
        use scancode::Scancode::*;
        match scancode {
            Q => self.move_camera(Point2D::new(-1, 0), 0),
            W => self.move_camera(Point2D::new(-1, -1), 0),
            E => self.move_camera(Point2D::new(0, -1), 0),
            A => self.move_camera(Point2D::new(0, 1), 0),
            S => self.move_camera(Point2D::new(1, 1), 0),
            D => self.move_camera(Point2D::new(1, 0), 0),
            F1 => self.switch_camera(),
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

/// Type for maps saved into disk.
#[derive(Debug, RustcEncodable, RustcDecodable)]
struct MapSave {
    pub map: String,
    pub legend: HashMap<char, LegendItem>,
}

impl fmt::Display for MapSave {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TOML formatted output.
        writeln!(f, "map = '''")?;
        for line in self.map.lines() {
            writeln!(f, "{}", line.trim_right())?;
        }
        writeln!(f, "'''\n")?;
        writeln!(f, "[legend]")?;
        for (k, v) in self.legend.iter() {
            writeln!(f, "\"{}\" = {}", k, v)?;
        }
        Ok(())
    }
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
struct LegendItem {
    /// Terrain
    pub t: String,
    /// Entities
    pub e: Vec<String>,
}

impl fmt::Display for LegendItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TOML formatted output.
        write!(f, "{{ t = \"{}\", e = [", self.t)?;
        self.e.iter().next().map_or(Ok(()), |e| write!(f, "\"{}\"", e))?;
        for e in self.e.iter().skip(1) {
            write!(f, ", \"{}\"", e)?;
        }
        write!(f, "] }}")
    }
}
