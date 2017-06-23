use calx_grid::Dir6;
use display;
use euclid::{Point2D, Rect, Size2D, vec2};
use rand;
use vitral::{Context, FracPoint2D, FracSize2D, FracRect, Align};
use scancode::Scancode;
use std::fs::File;
use std::io::prelude::*;
use world::{Command, Location, Slot, TerrainQuery, World, on_screen};

pub struct View {
    pub world: World,
    pub console: display::Console,
    pub console_is_large: bool,
    pub show_inventory: bool,
}

impl View {
    pub fn new(world: World) -> View {
        View {
            world,
            console: display::Console::default(),
            console_is_large: false,
            show_inventory: false,
        }
    }

    fn game_input(&mut self, scancode: Scancode) -> Result<(), ()> {
        use scancode::Scancode::*;
        match scancode {
            Tab => Ok(self.console_is_large = !self.console_is_large),
            Q => self.world.step(Dir6::Northwest),
            W => self.world.step(Dir6::North),
            E => self.world.step(Dir6::Northeast),
            A => self.world.step(Dir6::Southwest),
            S => self.world.step(Dir6::South),
            D => self.world.step(Dir6::Southeast),
            I => {
                self.show_inventory = !self.show_inventory;
                Ok(())
            }
            F5 => {
                self.world
                    .save(&mut File::create("save.gam").unwrap())
                    .unwrap();
                Ok(())
            }
            F9 => {
                let mut savefile = File::open("save.gam").unwrap();
                self.world = World::load(&mut savefile).unwrap();
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn console_input(&mut self, scancode: Scancode) -> Result<(), ()> {
        use scancode::Scancode::*;
        match scancode {
            Tab => Ok(self.console_is_large = !self.console_is_large),
            Enter | PadEnter => {
                let input = self.console.get_input();
                let _ = writeln!(&mut self.console, "{}", input);
                if let Err(e) = self.parse(&input) {
                    let _ = writeln!(&mut self.console, "{}", e);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn dump(&mut self) { dump_map(&self.world); }

    /// Generate a new random cave map.
    fn cave(&mut self) {
        use world::mapgen;
        self.world = World::new(1);
        mapgen::caves(&mut self.world,
                      &mut rand::thread_rng(),
                      Location::new(0, 0, 0),
                      300);
    }

    /// Generate a new random maze map.
    fn maze(&mut self, sparseness: usize) {
        use world::mapgen;
        self.world = World::new(1);
        mapgen::maze(&mut self.world, &mut rand::thread_rng(), sparseness);
    }

    /// Generate a new random rooms and corridors
    fn rooms(&mut self) {
        use world::mapgen;
        self.world = World::new(1);
        mapgen::rooms(&mut self.world, &mut rand::thread_rng());
    }

    command_parser!{
        fn cave(&mut self);
        fn maze(&mut self, sparseness: usize);
        fn rooms(&mut self);

        fn dump(&mut self);
    }

    fn draw_inventory(&mut self, c: &mut display::Backend, area: &Rect<f32>) {
        // Start with hardcoded invetory data to test the UI logic.
        c.fill_rect(FracRect::new(FracPoint2D::new(0.0, 0.0), FracSize2D::new(1.0, 1.0)),
                    [0.0, 0.0, 0.0, 0.99]);

        let mut letter_pos = Point2D::new(0.0, 0.0);
        let mut slot_name_pos = Point2D::new(20.0, 0.0);
        let mut item_name_pos = Point2D::new(80.0, 0.0);
        let text_color = [1.0, 1.0, 1.0, 1.0];

        for slot in SLOT_DATA.iter() {
            // TODO: Bounding box for these is a button...
            letter_pos = c.draw_text(letter_pos, Align::Left, text_color,
                                   &format!("{})", slot.key));
            slot_name_pos = c.draw_text(slot_name_pos, Align::Left, text_color, slot.name);
            item_name_pos = c.draw_text(item_name_pos, Align::Left, text_color, "[Inventory Item]");
        }
    }

    pub fn draw(&mut self, context: &mut display::Backend, screen_area: &Rect<f32>) {
        let camera_loc = Location::new(0, 0, 0);
        let mut view = display::WorldView::new(camera_loc, *screen_area);
        view.show_cursor = true;

        view.draw(&self.world, context);

        // Small console drawn as part of the main view, before the extra UI windows like
        // inventory.
        if !self.console_is_large {
            self.console.draw_small(context, screen_area);
        }

        if self.show_inventory {
            self.draw_inventory(context, &Rect::new(Point2D::new(0.0, 0.0), Size2D::new(320.0, 360.0)));
        }

        // Large console is drawn on top of all other windows.
        if self.console_is_large {
            let mut console_area = *screen_area;
            console_area.size.height /= 2.0;
            self.console.draw_large(context, &console_area);
        }

        if let Some(scancode) = context.poll_key().and_then(|k| Scancode::new(k.scancode)) {
            let _ = if self.console_is_large {
                self.console_input(scancode)
            } else {
                self.game_input(scancode)
            };
        }
    }
}

/// Print the world map as ASCII.
fn dump_map(world: &World) {
    for y in -21..21 {
        for x in -39..41 {
            if (x + y) % 2 != 0 {
                print!(" ");
                continue;
            }
            let pos = vec2((x + y) / 2, y);
            if on_screen(pos) {
                let t = world.terrain(Location::new(0, 0, 0) + pos);
                if t.is_open() {
                    print!(".");
                } else if t.is_door() {
                    print!("+");
                } else if t.is_wall() {
                    print!("#");
                } else {
                    print!("*");
                }
            } else {
                print!(" ");
            }
        }
        println!("");
    }
}

struct SlotData {
    key: char,
    slot: Slot,
    name: &'static str,
}

static SLOT_DATA: [SlotData; 34] = [
    SlotData { key: '1', slot: Slot::Spell1,     name: "Ability" },
    SlotData { key: '2', slot: Slot::Spell2,     name: "Ability" },
    SlotData { key: '3', slot: Slot::Spell3,     name: "Ability" },
    SlotData { key: '4', slot: Slot::Spell4,     name: "Ability" },
    SlotData { key: '5', slot: Slot::Spell5,     name: "Ability" },
    SlotData { key: '6', slot: Slot::Spell6,     name: "Ability" },
    SlotData { key: '7', slot: Slot::Spell7,     name: "Ability" },
    SlotData { key: '8', slot: Slot::Spell8,     name: "Ability" },
    SlotData { key: 'a', slot: Slot::Melee,      name: "Weapon" },
    SlotData { key: 'b', slot: Slot::Ranged,     name: "Ranged" },
    SlotData { key: 'c', slot: Slot::Head,       name: "Head" },
    SlotData { key: 'd', slot: Slot::Body,       name: "Body" },
    SlotData { key: 'e', slot: Slot::Feet,       name: "Feet" },
    SlotData { key: 'f', slot: Slot::TrinketF,   name: "Trinket" },
    SlotData { key: 'g', slot: Slot::TrinketG,   name: "Trinket" },
    SlotData { key: 'h', slot: Slot::TrinketH,   name: "Trinket" },
    SlotData { key: 'i', slot: Slot::TrinketI,   name: "Trinket" },
    SlotData { key: 'j', slot: Slot::InventoryJ, name: "" },
    SlotData { key: 'k', slot: Slot::InventoryK, name: "" },
    SlotData { key: 'l', slot: Slot::InventoryL, name: "" },
    SlotData { key: 'm', slot: Slot::InventoryM, name: "" },
    SlotData { key: 'n', slot: Slot::InventoryN, name: "" },
    SlotData { key: 'o', slot: Slot::InventoryO, name: "" },
    SlotData { key: 'p', slot: Slot::InventoryP, name: "" },
    SlotData { key: 'q', slot: Slot::InventoryQ, name: "" },
    SlotData { key: 'r', slot: Slot::InventoryR, name: "" },
    SlotData { key: 's', slot: Slot::InventoryS, name: "" },
    SlotData { key: 't', slot: Slot::InventoryT, name: "" },
    SlotData { key: 'u', slot: Slot::InventoryU, name: "" },
    SlotData { key: 'v', slot: Slot::InventoryV, name: "" },
    SlotData { key: 'w', slot: Slot::InventoryW, name: "" },
    SlotData { key: 'x', slot: Slot::InventoryX, name: "" },
    SlotData { key: 'y', slot: Slot::InventoryY, name: "" },
    SlotData { key: 'z', slot: Slot::InventoryZ, name: "" },
];
