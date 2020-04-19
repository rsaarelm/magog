use crate::msg;
use calx::{stego, CellVector, Dir6, IncrementalState};
use calx_ecs::Entity;
use display::{CanvasExt, ScreenVector};
use euclid::default::{Point2D, Rect};
use euclid::{point2, size2, vec2};
use std::io::prelude::*;
use std::io::Cursor;
use vitral::{
    color, Align, ButtonAction, Canvas, InputEvent, Keycode, RectUtil, Rgba, Scene, SceneSwitch,
};
use world::{Ability, ActionOutcome, Command, LerpLocation, Slot, World, WorldSeed};

pub struct HotbarAction {
    ability: Ability,
    // Is the ability activated via an item instead of being innate.
    item: Option<Entity>,
}

pub(crate) struct GameRuntime {
    world: IncrementalState<World>,
    command: Option<Command>,
    cursor_item: Option<Entity>,
    hotbar: [Option<HotbarAction>; 10],
    hotbar_focus: Option<usize>,
}

impl GameRuntime {
    pub fn new(seed: WorldSeed) -> GameRuntime {
        GameRuntime {
            world: IncrementalState::new(seed),
            command: None,
            cursor_item: None,
            hotbar: Default::default(),
            hotbar_focus: None,
        }
    }

    /// Method to force commands from eg. inventory mode
    pub fn force_command(&mut self, cmd: Command) -> bool {
        if !self.world.can_command(&cmd) {
            return false;
        }

        while self.world.player().is_some() && !self.world.player_can_act() {
            self.world.update(Command::Wait);
        }

        if self.world.player().is_some() {
            debug_assert!(self.world.player_can_act());
            // TODO FIXME Process events not getting called on events generated here.
            self.world.update(cmd);
        }
        true
    }

    fn is_bindable_hotbar_action(&self, slot: usize) -> bool {
        match self.hotbar[slot] {
            Some(HotbarAction { ability, .. }) => ability.is_targeted(),
            _ => false,
        }
    }

    fn is_untargeted_hotbar_action(&self, slot: usize) -> bool {
        match self.hotbar[slot] {
            Some(HotbarAction { ability, .. }) => !ability.is_targeted(),
            _ => false,
        }
    }

    pub fn draw_hotbar(&mut self, canvas: &mut Canvas) {
        for x in 0..10 {
            let pos = point2(204 + x as i32 * 24, 344);
            let bounds = Rect::new(pos, size2(16, 16));
            let color = if Some(x) == self.hotbar_focus {
                color::ORANGE
            } else {
                color::RED
            };
            canvas.fill_rect(&bounds.inflate(1, 1), color);
            canvas.fill_rect(&bounds, color::BLACK);

            match self.hotbar[x] {
                None => {}
                Some(HotbarAction {
                    item: Some(item), ..
                }) => {
                    canvas.draw_item_icon(
                        pos + vec2(8, 8),
                        self.world.entity_icon(item).expect("Item icon missing"),
                        self.world.count(item),
                    );
                }
                Some(HotbarAction {
                    ability: _ability, ..
                }) => {
                    // TODO: Icons for raw abilities
                }
            }

            if let Some(item) = self.cursor_item {
                match canvas.click_state(&bounds) {
                    ButtonAction::LeftClicked => {
                        // Bind a usable item into the hotbar
                        if let Some(&ability) = self.world.list_abilities(item).iter().next() {
                            self.hotbar[x] = Some(HotbarAction {
                                ability,
                                item: Some(item),
                            });

                            self.cursor_item = None;
                        }
                    }
                    _ => {}
                }
            } else {
                match canvas.click_state(&bounds) {
                    ButtonAction::LeftClicked if self.is_bindable_hotbar_action(x) => {
                        self.hotbar_focus = Some(x);
                    }
                    ButtonAction::RightClicked if self.is_bindable_hotbar_action(x) => {
                        self.hotbar_focus = Some(x);
                    }

                    // Right-click to immediately fire an untargeted action
                    ButtonAction::RightClicked if self.is_untargeted_hotbar_action(x) => {
                        if let Some(HotbarAction { ability, item }) = self.hotbar[x] {
                            self.force_command(Command::UntargetedAbility { ability, item });
                        }
                    }

                    ButtonAction::MiddleClicked => {
                        self.hotbar[x] = None;
                    }
                    _ => {}
                }
            }
        }
    }

    /// Handle missing items and invalidated abilities
    fn update_hotbar(&mut self) {
        for i in 0..self.hotbar.len() {
            if let Some(HotbarAction {
                item: Some(item), ..
            }) = self.hotbar[i]
            {
                if !self.world.is_alive(item) {
                    self.hotbar[i] = None;
                }
            }
        }
    }
}

#[derive(Default)]
pub struct GameLoop {
    pub console: display::Console,
    camera_loc: LerpLocation,
}

enum Side {
    West,
    East,
}

impl Scene<GameRuntime> for GameLoop {
    fn update(&mut self, ctx: &mut GameRuntime) -> Option<SceneSwitch<GameRuntime>> {
        ctx.update_hotbar();
        self.process_events(ctx);

        if ctx.world.player_can_act() {
            if let Some(cmd) = ctx.command {
                ctx.world.update(cmd);
                ctx.command = None;
            } else {
                ctx.world.tick_anims();
            }
        } else {
            // Not waiting for player input, do we speed up?
            let fast_forward_speed = if ctx.world.player().is_some() {
                if ctx.command.is_some() {
                    // Impatient player is already tapping the keys, time to really speed up.
                    30
                } else {
                    // Otherwise just move at a moderately snappy pace.
                    3
                }
            } else {
                // Don't fast forward when player is dead.
                1
            };

            for _ in 0..fast_forward_speed {
                if ctx.world.player_can_act() {
                    break;
                }
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
        ctx.update_hotbar();

        let screen_area = canvas.screen_bounds();

        let (view_area, status_area) = screen_area.horizontal_split(-32);

        if let Some(loc) = ctx
            .world
            .player()
            .map(|x| ctx.world.lerp_location(x).unwrap())
        {
            self.camera_loc = loc;
        }

        let mut view = display::WorldView::new(self.camera_loc, view_area);
        view.show_cursor = true;

        canvas.set_clip(view_area);
        view.draw(&*ctx.world, canvas);
        canvas.clear_clip();

        canvas.set_clip(status_area);
        self.status_draw(ctx, canvas, &status_area);
        canvas.clear_clip();

        let mut console_area = screen_area;
        console_area.size.height = 32;
        self.console.draw_small(canvas, &console_area);

        if view_area.contains(canvas.mouse_pos()) {
            let mouse_loc =
                view.screen_to_cell(ScreenVector::from_untyped(canvas.mouse_pos().to_vector()));
            (|| {
                let player = ctx.world.player()?;
                let relative_vec = ctx.world.location(player)?.v2_at(mouse_loc)?;
                let click_state = canvas.click_state(&view_area);

                if click_state == ButtonAction::LeftClicked {
                    if relative_vec == CellVector::zero() {
                        ctx.command = Some(Command::Take);
                    } else {
                        let dir = Dir6::from_v2(relative_vec);
                        self.smart_step(ctx, dir);
                    }
                }

                // Use targeted ability with RMB
                if click_state == ButtonAction::RightClicked {
                    if let Some(i) = ctx.hotbar_focus {
                        if let Some(HotbarAction { ability, item }) = ctx.hotbar[i] {
                            if relative_vec != CellVector::zero() {
                                let dir = Dir6::from_v2(relative_vec);
                                ctx.command = Some(Command::TargetedAbility { ability, dir, item });
                            }
                        }
                    }
                }
                Some(())
            })();
        }

        None
    }

    fn input(
        &mut self,
        ctx: &mut GameRuntime,
        event: &InputEvent,
        canvas: &mut Canvas,
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

                // XXX: Wizard mode key, disable in legit gameplay mode
                Backspace => {
                    ctx.world.edit_history(|history| {
                        // Find the last non-Wait command and cut off before that.
                        if let Some((idx, _)) = history
                            .events
                            .iter()
                            .enumerate()
                            .rev()
                            .find(|(_, &c)| c != Command::Wait)
                        {
                            println!("DEBUG Undoing last turn");
                            history.events.truncate(idx);
                        }
                    });
                }

                G => {
                    ctx.command = Some(Command::Take);
                }

                Escape => {
                    return Some(SceneSwitch::Push(Box::new(InventoryScreen)));
                }
                F5 => {
                    // Quick save.

                    let enc = ron::ser::to_string_pretty(&ctx.world, Default::default()).unwrap();
                    let cover = canvas.screenshot();
                    let save = stego::embed_gzipped(&cover, enc.as_bytes());
                    let _ = image::save_buffer(
                        "save.png",
                        &save,
                        save.width(),
                        save.height(),
                        image::ColorType::Rgb8,
                    );
                }
                F9 => {
                    // Quick load

                    // TODO: Error handling when file is missing or not an image.
                    let save = image::open("save.png").unwrap().to_rgb();
                    // TODO: Error handling when stego data can't be retrieved
                    let save = stego::extract(&save).unwrap();
                    // TODO: Error handling when stego data can't be deserialized into world
                    let new_world: IncrementalState<World> =
                        ron::de::from_reader(&mut Cursor::new(&save)).unwrap();
                    ctx.world = new_world;
                }
                F12 => {
                    // Capture screenshot.
                    let shot = canvas.screenshot();
                    let _ = calx::save_screenshot("magog", &shot);
                }

                _ => {}
            }
        }
        None
    }
}

impl GameLoop {
    /// Step command that turns into melee attack if an enemy is in the way.
    fn smart_step(&self, ctx: &mut GameRuntime, dir: Dir6) -> ActionOutcome {
        let player = ctx.world.player()?;
        let loc = ctx.world.location(player)?;

        // Wall slide
        let dir = {
            let (left, fwd, right) = (
                ctx.world.can_step_on_terrain(player, dir - 1),
                ctx.world.can_step_on_terrain(player, dir),
                ctx.world.can_step_on_terrain(player, dir + 1),
            );
            if !fwd && left {
                dir - 1
            } else if !fwd && right {
                dir + 1
            } else {
                dir
            }
        };

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
        Some(true)
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

    fn status_draw(&self, ctx: &mut GameRuntime, canvas: &mut Canvas, area: &Rect<i32>) {
        canvas.fill_rect(area, Rgba::from(0x33_11_11_ff));
        canvas.draw_text(
            &*display::font(),
            area.origin,
            Align::Left,
            color::RED,
            "Welcome to status bar",
        );

        ctx.draw_hotbar(canvas);
    }

    fn process_events(&mut self, _ctx: &mut GameRuntime) {
        for msg in &msg::get() {
            let _ = writeln!(&mut self.console, "{}", msg);
        }
    }
}

struct InventoryScreen;

enum PickAction {
    Pick(Entity),
    Place(Entity),
    Swap(Entity, Entity),
    Drop(Entity),
}

impl Scene<GameRuntime> for InventoryScreen {
    fn render(
        &mut self,
        ctx: &mut GameRuntime,
        canvas: &mut Canvas,
    ) -> Option<SceneSwitch<GameRuntime>> {
        use PickAction::*;

        fn handle_action(ctx: &mut GameRuntime, slot: Slot, action: Option<PickAction>) {
            match action {
                Some(Pick(e)) => {
                    ctx.cursor_item = Some(e);
                }
                Some(Place(e)) => {
                    // Putting it back where you took it, no-op but change UI.
                    if let Some(old_slot) = ctx.world.entity_slot(e) {
                        if old_slot == slot {
                            ctx.cursor_item = None;
                        }
                    }

                    // Put in new slot, emit command
                    if ctx.force_command(Command::InventoryPlace(e, slot)) {
                        ctx.cursor_item = None;
                    }
                }
                Some(Swap(current, new)) => {
                    ctx.cursor_item = Some(new);
                    if let Some(old_slot) = ctx.world.entity_slot(current) {
                        if ctx.force_command(Command::InventorySwap(old_slot, slot)) {
                            ctx.cursor_item = Some(new);
                        }
                    }
                }
                Some(Drop(_e)) => {
                    ctx.force_command(Command::Drop(slot));
                }
                _ => {}
            }
        }

        // Inventory items
        for y in 0..5 {
            for x in 0..10 {
                let pos = point2(8 + x * 24, 8 + y * 24);
                let bounds = Rect::new(pos, size2(16, 16));
                canvas.fill_rect(&bounds.inflate(1, 1), color::GREEN);
                canvas.fill_rect(&bounds, color::BLACK);

                let slot = Slot::Bag((x + y * 10) as u32);

                let action = self.item_button(ctx, canvas, pos, slot);
                handle_action(ctx, slot, action);
            }
        }

        // Equipment
        for (i, &slot) in [
            Slot::Trinket1,
            Slot::Head,
            Slot::Ranged,
            Slot::RightHand,
            Slot::Body,
            Slot::LeftHand,
            Slot::Trinket2,
            Slot::Feet,
            Slot::Trinket3,
        ]
        .iter()
        .enumerate()
        {
            let (x, y) = (i as i32 % 3, i as i32 / 3);
            let pos = point2(256 + x * 24, 8 + y * 24);
            let bounds = Rect::new(pos, size2(16, 16));
            canvas.fill_rect(&bounds.inflate(1, 1), color::SILVER);
            canvas.fill_rect(&bounds, color::BLACK);

            let action = self.item_button(ctx, canvas, pos, slot);
            handle_action(ctx, slot, action);
        }

        ctx.draw_hotbar(canvas);

        // Draw cursor item as cursor
        if let Some(item) = ctx.cursor_item {
            let pos = canvas.mouse_pos();
            canvas.draw_item_icon(
                pos,
                ctx.world.entity_icon(item).expect("Item icon missing"),
                ctx.world.count(item),
            );
        }
        None
    }

    fn input(
        &mut self,
        ctx: &mut GameRuntime,
        event: &InputEvent,
        _canvas: &mut Canvas,
    ) -> Option<SceneSwitch<GameRuntime>> {
        if let InputEvent::KeyEvent {
            is_down: true,
            hardware_key: Some(scancode),
            ..
        } = event
        {
            use Keycode::*;
            match scancode {
                Escape => {
                    ctx.cursor_item = None;
                    return Some(SceneSwitch::Pop);
                }
                _ => {}
            }
        }
        None
    }
}

impl InventoryScreen {
    /// Return entity if item was clicked and grabbed.
    fn item_button(
        &self,
        ctx: &mut GameRuntime,
        canvas: &mut Canvas,
        pos: Point2D<i32>,
        slot: Slot,
    ) -> Option<PickAction> {
        let item: Option<Entity> = (|| ctx.world.entity_equipped(ctx.world.player()?, slot))();
        let bounds = Rect::new(pos, size2(16, 16));

        if item != ctx.cursor_item {
            if let Some(e) = item {
                canvas.draw_item_icon(
                    pos + vec2(8, 8),
                    ctx.world.entity_icon(e).expect("Item icon missing"),
                    ctx.world.count(e),
                );
                if canvas.click_state(&bounds) == ButtonAction::LeftClicked {
                    return match ctx.cursor_item {
                        None => Some(PickAction::Pick(e)),
                        Some(c) => Some(PickAction::Swap(c, e)),
                    };
                }
                if canvas.click_state(&bounds) == ButtonAction::MiddleClicked {
                    if ctx.cursor_item.is_none() {
                        return Some(PickAction::Drop(e));
                    }
                }
            }
        }

        if let Some(item) = ctx.cursor_item {
            if canvas.click_state(&bounds) == ButtonAction::LeftClicked {
                return Some(PickAction::Place(item));
            }
        }

        None
    }
}
