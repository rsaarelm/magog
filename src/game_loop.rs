use calx_grid::Dir6;
use display;
use euclid::{Point2D, Rect, vec2};
use rand;
use scancode::Scancode;
use std::fs::File;
use std::io::prelude::*;
use vitral::{Context, FracPoint2D, FracSize2D, FracRect, Align};
use world::{Command, CommandResult, Event, ItemType, Location, Query, Slot, TerrainQuery, World, on_screen};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum State {
    Main,
    Inventory(InventoryAction),
    Console,
    Aim(AimAction),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum InventoryAction {
    Drop,
    Equip,
    Use,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum AimAction {
    Zap(Slot),
    // Maybe add intrinsic abilities not tied to a specific entity later
}

pub struct GameLoop {
    pub world: World,
    pub console: display::Console,
    state: State,
}

impl GameLoop {
    pub fn new(world: World) -> GameLoop {
        GameLoop {
            world,
            console: display::Console::default(),
            state: State::Main,
        }
    }

    fn game_input(&mut self, scancode: Scancode) -> CommandResult {
        use scancode::Scancode::*;
        match scancode {
            Tab => {
                self.state = State::Console;
                Ok(Vec::new())
            }
            Q => self.world.step(Dir6::Northwest),
            W => self.world.step(Dir6::North),
            E => self.world.step(Dir6::Northeast),
            A => self.world.step(Dir6::Southwest),
            S => self.world.step(Dir6::South),
            D => self.world.step(Dir6::Southeast),
            I => {
                self.state = State::Inventory(InventoryAction::Equip);
                Ok(Vec::new())
            }
            // XXX: Key mnemonics, bit awkward when D is taken by movement.
            B => {
                self.state = State::Inventory(InventoryAction::Drop);
                Ok(Vec::new())
            }
            U => {
                self.state = State::Inventory(InventoryAction::Use);
                Ok(Vec::new())
            }
            G => self.world.take(),
            F5 => {
                self.world
                    .save(&mut File::create("save.gam").unwrap())
                    .unwrap();
                Ok(Vec::new())
            }
            F9 => {
                let mut savefile = File::open("save.gam").unwrap();
                self.world = World::load(&mut savefile).unwrap();
                Ok(Vec::new())
            }
            _ => Ok(Vec::new()),
        }
    }

    fn aim_input(&mut self, slot: Slot, scancode: Scancode) -> CommandResult {
        use scancode::Scancode::*;
        match scancode {
            Q => self.world.zap_item(slot, Dir6::Northwest),
            W => self.world.zap_item(slot, Dir6::North),
            E => self.world.zap_item(slot, Dir6::Northeast),
            A => self.world.zap_item(slot, Dir6::Southwest),
            S => self.world.zap_item(slot, Dir6::South),
            D => self.world.zap_item(slot, Dir6::Southeast),
            Escape => {
                self.state = State::Main;
                Ok(Vec::new())
            }
            _ => Ok(Vec::new()),
        }
    }

    fn inventory_input(&mut self, scancode: Scancode) -> CommandResult {
        use scancode::Scancode::*;
        for slot in SLOT_DATA.iter() {
            if scancode == slot.code {
                if let State::Inventory(action) = self.state {
                    let ret = self.inventory_action(slot.slot, action);
                    if ret.is_ok() {
                        return ret;
                    }
                }
            }
        }

        match scancode {
            Escape => {
                self.state = State::Main;
                Ok(Vec::new())
            }
            _ => Ok(Vec::new()),
        }
    }

    fn inventory_action(&mut self, slot: Slot, action: InventoryAction) -> CommandResult {
        match action {
            InventoryAction::Drop => {
                let ret = self.world.drop(slot);
                // After succesful drop, go back to main state.
                if ret.is_ok() {
                    self.state = State::Main;
                }
                ret
            }
            // Can equip multiple items in one go, wait for ESC to return to main state.
            InventoryAction::Equip => self.world.equip(slot),
            InventoryAction::Use => {
                let player = self.world.player().ok_or(())?;

                if let Some(item) = self.world.entity_equipped(player, slot) {
                    match self.world.item_type(item) {
                        Some(ItemType::UntargetedUsable(_)) => {
                            return self.world.use_item(slot);
                        }
                        Some(ItemType::TargetedUsable(_)) => {
                            // If we need to aim, switch to aim state before calling world.
                            self.state = State::Aim(AimAction::Zap(slot));
                            return Ok(Vec::new());
                        }
                        _ => {}
                    }
                }
                return Err(());
            }
        }
    }

    fn console_input(&mut self, scancode: Scancode) -> CommandResult {
        use scancode::Scancode::*;
        match scancode {
            Tab => {
                self.state = State::Main;
                Ok(Vec::new())
            }
            Enter | PadEnter => {
                let input = self.console.get_input();
                let _ = writeln!(&mut self.console, "{}", input);
                if let Err(e) = self.parse(&input) {
                    let _ = writeln!(&mut self.console, "{}", e);
                }
                Ok(Vec::new())
            }
            _ => Ok(Vec::new()),
        }
    }

    fn dump(&mut self) { dump_map(&self.world); }

    /// Generate a new random cave map.
    fn cave(&mut self) {
        use world::mapgen;
        self.world = World::new(1);
        mapgen::caves(
            &mut self.world,
            &mut rand::thread_rng(),
            Location::new(0, 0, 0),
            300,
        );
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

    fn draw_inventory(&mut self, c: &mut display::Backend) -> Result<(), ()> {
        let player = self.world.player().ok_or(())?;

        // Start with hardcoded invetory data to test the UI logic.
        c.fill_rect(
            FracRect::new(FracPoint2D::new(0.0, 0.0), FracSize2D::new(1.0, 1.0)),
            [0.0, 0.0, 0.0, 0.99],
        );

        let mut letter_pos = Point2D::new(0.0, 0.0);
        let mut slot_name_pos = Point2D::new(20.0, 0.0);
        let mut item_name_pos = Point2D::new(80.0, 0.0);
        let text_color = [1.0, 1.0, 1.0, 1.0];

        for slot in SLOT_DATA.iter() {
            // TODO: Bounding box for these is a button...
            letter_pos = c.draw_text(
                letter_pos,
                Align::Left,
                text_color,
                &format!("{})", slot.key),
            );
            slot_name_pos = c.draw_text(slot_name_pos, Align::Left, text_color, slot.name);
            let item_name = if let Some(item) = self.world.entity_equipped(player, slot.slot) {
                self.world.entity_name(item)
            } else {
                "".to_string()
            };

            item_name_pos = c.draw_text(item_name_pos, Align::Left, text_color, &item_name);
        }

        Ok(())
    }

    pub fn draw(&mut self, context: &mut display::Backend, screen_area: &Rect<f32>) {
        let camera_loc = Location::new(0, 0, 0);
        let mut view = display::WorldView::new(camera_loc, *screen_area);
        view.show_cursor = true;

        view.draw(&self.world, context);

        match self.state {
            State::Inventory(_) => {
                let _ = self.draw_inventory(context);
            }
            State::Console => {
                let mut console_area = *screen_area;
                console_area.size.height /= 2.0;
                self.console.draw_large(context, &console_area);
            }
            _ => {
                self.console.draw_small(context, screen_area);
            }
        }

        if let Some(scancode) = context.poll_key().and_then(|k| Scancode::new(k.scancode)) {
            let ret = match self.state {
                State::Inventory(_) => self.inventory_input(scancode),
                State::Console => self.console_input(scancode),
                State::Aim(AimAction::Zap(slot)) => self.aim_input(slot, scancode),
                _ => self.game_input(scancode),
            };

            if let Ok(events) = ret {
                // Input event caused a successful world step and we got an event sequence out.
                // Convert events into UI display effects.
                for e in events {
                    match e {
                        Event::Msg(text) => {
                            let _ = writeln!(&mut self.console, "{}", text);
                        }
                    }
                }
            }
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
    code: Scancode,
    slot: Slot,
    name: &'static str,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
static SLOT_DATA: [SlotData; 34] = [
    SlotData { key: '1', code: Scancode::Num1, slot: Slot::Spell1,     name: "Ability" },
    SlotData { key: '2', code: Scancode::Num2, slot: Slot::Spell2,     name: "Ability" },
    SlotData { key: '3', code: Scancode::Num3, slot: Slot::Spell3,     name: "Ability" },
    SlotData { key: '4', code: Scancode::Num4, slot: Slot::Spell4,     name: "Ability" },
    SlotData { key: '5', code: Scancode::Num5, slot: Slot::Spell5,     name: "Ability" },
    SlotData { key: '6', code: Scancode::Num6, slot: Slot::Spell6,     name: "Ability" },
    SlotData { key: '7', code: Scancode::Num7, slot: Slot::Spell7,     name: "Ability" },
    SlotData { key: '8', code: Scancode::Num8, slot: Slot::Spell8,     name: "Ability" },
    SlotData { key: 'a', code: Scancode::A,    slot: Slot::Melee,      name: "Weapon" },
    SlotData { key: 'b', code: Scancode::B,    slot: Slot::Ranged,     name: "Ranged" },
    SlotData { key: 'c', code: Scancode::C,    slot: Slot::Head,       name: "Head" },
    SlotData { key: 'd', code: Scancode::D,    slot: Slot::Body,       name: "Body" },
    SlotData { key: 'e', code: Scancode::E,    slot: Slot::Feet,       name: "Feet" },
    SlotData { key: 'f', code: Scancode::F,    slot: Slot::TrinketF,   name: "Trinket" },
    SlotData { key: 'g', code: Scancode::G,    slot: Slot::TrinketG,   name: "Trinket" },
    SlotData { key: 'h', code: Scancode::H,    slot: Slot::TrinketH,   name: "Trinket" },
    SlotData { key: 'i', code: Scancode::I,    slot: Slot::TrinketI,   name: "Trinket" },
    SlotData { key: 'j', code: Scancode::J,    slot: Slot::InventoryJ, name: "" },
    SlotData { key: 'k', code: Scancode::K,    slot: Slot::InventoryK, name: "" },
    SlotData { key: 'l', code: Scancode::L,    slot: Slot::InventoryL, name: "" },
    SlotData { key: 'm', code: Scancode::M,    slot: Slot::InventoryM, name: "" },
    SlotData { key: 'n', code: Scancode::N,    slot: Slot::InventoryN, name: "" },
    SlotData { key: 'o', code: Scancode::O,    slot: Slot::InventoryO, name: "" },
    SlotData { key: 'p', code: Scancode::P,    slot: Slot::InventoryP, name: "" },
    SlotData { key: 'q', code: Scancode::Q,    slot: Slot::InventoryQ, name: "" },
    SlotData { key: 'r', code: Scancode::R,    slot: Slot::InventoryR, name: "" },
    SlotData { key: 's', code: Scancode::S,    slot: Slot::InventoryS, name: "" },
    SlotData { key: 't', code: Scancode::T,    slot: Slot::InventoryT, name: "" },
    SlotData { key: 'u', code: Scancode::U,    slot: Slot::InventoryU, name: "" },
    SlotData { key: 'v', code: Scancode::V,    slot: Slot::InventoryV, name: "" },
    SlotData { key: 'w', code: Scancode::W,    slot: Slot::InventoryW, name: "" },
    SlotData { key: 'x', code: Scancode::X,    slot: Slot::InventoryX, name: "" },
    SlotData { key: 'y', code: Scancode::Y,    slot: Slot::InventoryY, name: "" },
    SlotData { key: 'z', code: Scancode::Z,    slot: Slot::InventoryZ, name: "" },
];
