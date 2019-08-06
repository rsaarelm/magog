use calx::{Dir6, IncrementalState};
use display;
use euclid::default::{Point2D, Rect};
use std::io::prelude::*;
use vitral::{self, color, Align, Canvas, InputEvent, Keycode, RectUtil, Rgba, Scene, SceneSwitch};
use world::{ActionOutcome, Command, Event, ItemType, Location, Mutate, Query, Slot, World};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum InventoryMode {
    Drop,
    Equip,
    Use,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum AimMode {
    Zap(Slot),
    // Maybe add intrinsic abilities not tied to a specific entity later
}

pub(crate) struct GameRuntime {
    world: IncrementalState<World>,
    command: Option<Command>,
}

impl GameRuntime {
    pub fn new(seed: u32) -> GameRuntime {
        GameRuntime {
            world: IncrementalState::new(seed),
            command: None,
        }
    }
}

pub struct GameLoop {
    pub console: display::Console,
    camera_loc: Location,
}

enum Side {
    West,
    East,
}

impl AimMode {
    fn act(&self, ctx: &mut GameRuntime, dir: Dir6) -> Option<SceneSwitch<GameRuntime>> {
        match self {
            AimMode::Zap(slot) => {
                ctx.command = Some(Command::Zap(*slot, dir));
                Some(SceneSwitch::Pop)
            }
        }
    }
}

impl Scene<GameRuntime> for AimMode {
    fn input(
        &mut self,
        ctx: &mut GameRuntime,
        event: InputEvent,
    ) -> Option<SceneSwitch<GameRuntime>> {
        if let InputEvent::KeyEvent {
            is_down: true,
            hardware_key: Some(scancode),
            ..
        } = event
        {
            use Keycode::*;
            match scancode {
                Q | Pad7 | Home => return self.act(ctx, Dir6::Northwest),
                W | Up | Pad8 => return self.act(ctx, Dir6::North),
                E | Pad9 | PageUp => return self.act(ctx, Dir6::Northeast),
                A | Pad1 | End => return self.act(ctx, Dir6::Southwest),
                S | Down | Pad2 => return self.act(ctx, Dir6::South),
                D | Pad3 | PageDown => return self.act(ctx, Dir6::Southeast),
                Escape => {
                    return Some(SceneSwitch::Pop);
                }
                _ => {}
            }
        }
        None
    }

    fn draw_previous(&self) -> bool { true }
}

impl Scene<GameRuntime> for InventoryMode {
    fn render(
        &mut self,
        ctx: &mut GameRuntime,
        canvas: &mut Canvas,
    ) -> Option<SceneSwitch<GameRuntime>> {
        let player = match ctx.world.player() {
            Some(p) => p,
            None => return Some(SceneSwitch::Pop),
        };

        let (_, bounds) = canvas.bounds().vertical_split(320);
        canvas.fill_rect(&bounds, [0.0, 0.0, 0.0, 0.99]);

        let mut letter_pos = Point2D::new(0, 0);
        let mut slot_name_pos = Point2D::new(20, 0);
        let mut item_name_pos = Point2D::new(80, 0);
        let text_color = [1.0, 1.0, 1.0, 1.0];

        for slot in SLOT_DATA.iter() {
            // TODO: Bounding box for these is a button...
            letter_pos = canvas.draw_text(
                &*display::font(),
                letter_pos,
                Align::Left,
                text_color,
                &format!("{})", slot.key),
            );
            slot_name_pos = canvas.draw_text(
                &*display::font(),
                slot_name_pos,
                Align::Left,
                text_color,
                slot.name,
            );
            let item_name = if let Some(item) = ctx.world.entity_equipped(player, slot.slot) {
                ctx.world.entity_name(item)
            } else {
                "".to_string()
            };

            item_name_pos = canvas.draw_text(
                &*display::font(),
                item_name_pos,
                Align::Left,
                text_color,
                &item_name,
            );
        }

        None
    }

    fn input(
        &mut self,
        ctx: &mut GameRuntime,
        event: InputEvent,
    ) -> Option<SceneSwitch<GameRuntime>> {
        if let InputEvent::KeyEvent {
            is_down: true,
            hardware_key: Some(scancode),
            ..
        } = event
        {
            use Keycode::*;

            for slot in SLOT_DATA.iter() {
                if scancode == slot.code {
                    match self {
                        InventoryMode::Drop => ctx.command = Some(Command::Drop(slot.slot)),
                        InventoryMode::Equip => ctx.command = Some(Command::Equip(slot.slot)),
                        InventoryMode::Use => {
                            // Need to see what happens when you use it.
                            let player = ctx.world.player()?;
                            let item = ctx.world.entity_equipped(player, slot.slot)?;

                            match ctx.world.item_type(item) {
                                Some(ItemType::UntargetedUsable(_)) => {
                                    // No further input needed, just fire off the command
                                    ctx.command = Some(Command::UseItem(slot.slot));
                                }
                                Some(ItemType::TargetedUsable(_)) => {
                                    // Items needs aiming, switch to aim mode.
                                    return Some(SceneSwitch::Replace(Box::new(AimMode::Zap(
                                        slot.slot,
                                    ))));
                                }
                                _ => {}
                            }
                        }
                    }
                    return Some(SceneSwitch::Pop);
                }
            }

            match scancode {
                Escape => {
                    return Some(SceneSwitch::Pop);
                }
                _ => {}
            }
        }
        None
    }

    fn draw_previous(&self) -> bool { true }
}

impl Scene<GameRuntime> for GameLoop {
    fn update(&mut self, ctx: &mut GameRuntime) -> Option<SceneSwitch<GameRuntime>> {
        if ctx.world.player_can_act() {
            if let Some(cmd) = ctx.command {
                ctx.world.update(cmd);
                ctx.command = None;
                for e in ctx.world.events() {
                    match e {
                        Event::Msg(text) => {
                            let _ = writeln!(&mut self.console, "{}", text);
                        }
                        Event::Damage { entity, amount } => {
                            let name = ctx.world.entity_name(*entity);
                            // TODO: Use graphical effect
                            let _ = writeln!(&mut self.console, "{} dmg {}", name, amount);
                        }
                    }
                }
            } else {
                ctx.world.tick_anims();
            }
        } else {
            // When playing turn-based and running the animations between player's inputs, speed
            // things up so that the pace feels snappy.
            const FAST_FORWARD: usize = 3;

            for _ in 0..FAST_FORWARD {
                if ctx.world.player_can_act() {
                    break;
                }
                // TODO FIXME process events in return value.
                ctx.world.update(Command::Wait);
            }
        }

        None
    }

    fn render(
        &mut self,
        ctx: &mut GameRuntime,
        canvas: &mut Canvas,
    ) -> Option<SceneSwitch<GameRuntime>> {
        let screen_area = canvas.screen_bounds();

        let (view_area, status_area) = screen_area.horizontal_split(-32);

        // Ugh
        ctx.world
            .player()
            .map(|x| ctx.world.location(x).map(|l| self.camera_loc = l));

        let mut view = display::WorldView::new(self.camera_loc, view_area);
        view.show_cursor = true;

        canvas.set_clip(view_area);
        view.draw(&*ctx.world, canvas);
        canvas.clear_clip();

        canvas.set_clip(status_area);
        self.status_draw(canvas, &status_area);
        canvas.clear_clip();

        let mut console_area = screen_area;
        console_area.size.height = 32;
        self.console.draw_small(canvas, &console_area);

        None
    }

    fn input(
        &mut self,
        ctx: &mut GameRuntime,
        event: InputEvent,
    ) -> Option<SceneSwitch<GameRuntime>> {
        if let InputEvent::KeyEvent {
            is_down: true,
            hardware_key: Some(scancode),
            ..
        } = event
        {
            use Keycode::*;

            match scancode {
                Q | Pad7 | Home => {
                    self.smart_step(ctx, Dir6::Northwest);
                }
                W | Up | Pad8 => {
                    self.smart_step(ctx, Dir6::North);
                }
                E | Pad9 | PageUp => {
                    self.smart_step(ctx, Dir6::Northeast);
                }
                A | Pad1 | End => {
                    self.smart_step(ctx, Dir6::Southwest);
                }
                S | Down | Pad2 => {
                    self.smart_step(ctx, Dir6::South);
                }
                D | Pad3 | PageDown => {
                    self.smart_step(ctx, Dir6::Southeast);
                }
                Left | Pad4 => {
                    self.side_step(ctx, Side::West);
                }
                Right | Pad6 => {
                    self.side_step(ctx, Side::East);
                }
                Space | Pad5 => {
                    ctx.command = Some(Command::Pass);
                }

                G => {
                    ctx.command = Some(Command::Take);
                }

                I => {
                    return Some(SceneSwitch::Push(Box::new(InventoryMode::Equip)));
                }
                B => {
                    return Some(SceneSwitch::Push(Box::new(InventoryMode::Drop)));
                }
                U => {
                    return Some(SceneSwitch::Push(Box::new(InventoryMode::Use)));
                }

                _ => {}
            }
        }
        None
    }
}

impl GameLoop {
    pub fn new() -> GameLoop {
        GameLoop {
            console: display::Console::new(display::font()),
            camera_loc: Location::new(0, 0, 0),
        }
    }

    /// Step command that turns into melee attack if an enemy is in the way.
    fn smart_step(&self, ctx: &mut GameRuntime, dir: Dir6) -> ActionOutcome {
        let player = ctx.world.player()?;
        let loc = ctx.world.location(player)?;
        let destination = loc.jump(&*ctx.world, dir);

        if let Some(mob) = ctx.world.mob_at(destination) {
            if ctx.world.is_hostile_to(player, mob) {
                // Fight on!
                ctx.command = Some(Command::Melee(dir));
            } else {
                // Do we want to do something smarter than walk into friendlies?
                // The world might treat this as a displace action so keep it like this for now.
                ctx.command = Some(Command::Step(dir));
            }
        } else {
            ctx.command = Some(Command::Step(dir));
        }
        Some(())
    }

    fn side_step(&self, ctx: &mut GameRuntime, side: Side) -> ActionOutcome {
        let player = ctx.world.player()?;
        let loc = ctx.world.location(player)?;
        let flip = (loc.x + loc.y) % 2 == 0;

        let actual_dir = match side {
            Side::West => {
                if flip {
                    Dir6::Southwest
                } else {
                    Dir6::Northwest
                }
            }
            Side::East => {
                if flip {
                    Dir6::Southeast
                } else {
                    Dir6::Northeast
                }
            }
        };

        self.smart_step(ctx, actual_dir)
    }

    pub fn status_draw(&self, canvas: &mut Canvas, area: &Rect<i32>) {
        canvas.fill_rect(area, Rgba::from(0x33_11_11_ff).into());
        canvas.draw_text(
            &*display::font(),
            area.origin,
            Align::Left,
            color::RED.into(),
            "Welcome to status bar",
        );
    }
}

struct SlotData {
    key: char,
    code: Keycode,
    slot: Slot,
    name: &'static str,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
static SLOT_DATA: [SlotData; 34] = [
    SlotData { key: '1', code: Keycode::Num1, slot: Slot::Spell1,     name: "Ability" },
    SlotData { key: '2', code: Keycode::Num2, slot: Slot::Spell2,     name: "Ability" },
    SlotData { key: '3', code: Keycode::Num3, slot: Slot::Spell3,     name: "Ability" },
    SlotData { key: '4', code: Keycode::Num4, slot: Slot::Spell4,     name: "Ability" },
    SlotData { key: '5', code: Keycode::Num5, slot: Slot::Spell5,     name: "Ability" },
    SlotData { key: '6', code: Keycode::Num6, slot: Slot::Spell6,     name: "Ability" },
    SlotData { key: '7', code: Keycode::Num7, slot: Slot::Spell7,     name: "Ability" },
    SlotData { key: '8', code: Keycode::Num8, slot: Slot::Spell8,     name: "Ability" },
    SlotData { key: 'a', code: Keycode::A,    slot: Slot::Melee,      name: "Weapon" },
    SlotData { key: 'b', code: Keycode::B,    slot: Slot::Ranged,     name: "Ranged" },
    SlotData { key: 'c', code: Keycode::C,    slot: Slot::Head,       name: "Head" },
    SlotData { key: 'd', code: Keycode::D,    slot: Slot::Body,       name: "Body" },
    SlotData { key: 'e', code: Keycode::E,    slot: Slot::Feet,       name: "Feet" },
    SlotData { key: 'f', code: Keycode::F,    slot: Slot::TrinketF,   name: "Trinket" },
    SlotData { key: 'g', code: Keycode::G,    slot: Slot::TrinketG,   name: "Trinket" },
    SlotData { key: 'h', code: Keycode::H,    slot: Slot::TrinketH,   name: "Trinket" },
    SlotData { key: 'i', code: Keycode::I,    slot: Slot::TrinketI,   name: "Trinket" },
    SlotData { key: 'j', code: Keycode::J,    slot: Slot::InventoryJ, name: "" },
    SlotData { key: 'k', code: Keycode::K,    slot: Slot::InventoryK, name: "" },
    SlotData { key: 'l', code: Keycode::L,    slot: Slot::InventoryL, name: "" },
    SlotData { key: 'm', code: Keycode::M,    slot: Slot::InventoryM, name: "" },
    SlotData { key: 'n', code: Keycode::N,    slot: Slot::InventoryN, name: "" },
    SlotData { key: 'o', code: Keycode::O,    slot: Slot::InventoryO, name: "" },
    SlotData { key: 'p', code: Keycode::P,    slot: Slot::InventoryP, name: "" },
    SlotData { key: 'q', code: Keycode::Q,    slot: Slot::InventoryQ, name: "" },
    SlotData { key: 'r', code: Keycode::R,    slot: Slot::InventoryR, name: "" },
    SlotData { key: 's', code: Keycode::S,    slot: Slot::InventoryS, name: "" },
    SlotData { key: 't', code: Keycode::T,    slot: Slot::InventoryT, name: "" },
    SlotData { key: 'u', code: Keycode::U,    slot: Slot::InventoryU, name: "" },
    SlotData { key: 'v', code: Keycode::V,    slot: Slot::InventoryV, name: "" },
    SlotData { key: 'w', code: Keycode::W,    slot: Slot::InventoryW, name: "" },
    SlotData { key: 'x', code: Keycode::X,    slot: Slot::InventoryX, name: "" },
    SlotData { key: 'y', code: Keycode::Y,    slot: Slot::InventoryY, name: "" },
    SlotData { key: 'z', code: Keycode::Z,    slot: Slot::InventoryZ, name: "" },
];
